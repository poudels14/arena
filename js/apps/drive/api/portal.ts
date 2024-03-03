import { uniq, uniqBy, keyBy, groupBy } from "lodash-es";
import ky from "ky";
import { z } from "zod";
import deepEqual from "fast-deep-equal/es6";
import { Search, searchRequestSchema } from "@portal/workspace-sdk/llm/search";
import { p } from "./procedure";
import { File } from "./repo/files";

const llmSearch = p
  .input(searchRequestSchema)
  .mutate(async ({ ctx, body, searchParams }): Promise<Search.Response> => {
    // TODO: idor check
    const sharedFilesSearchResults: any[] = await new Promise(
      async (resolve) => {
        if (
          searchParams.allApps != "true" ||
          // don't search other apps if the breadcrum isn't empty
          (body.context?.breadcrumbs || []).length > 0
        ) {
          return resolve([]);
        }
        const acls = await ky
          .get(
            `${ctx.workspaceHost}/api/acls?appTemplateId=${ctx.app.template.id}&userId=${ctx.user.id}`
          )
          .json<any[]>();

        const appIds = uniq(acls.map((acl) => acl.appId)).filter(
          (appId) => appId != ctx.app.id
        );

        const results = await Promise.all(
          appIds.map(async (appId) => {
            try {
              return await ky
                .post(
                  `${ctx.workspaceHost}/w/apps/${appId}/api/portal/llm/search`,
                  {
                    json: body,
                  }
                )
                .json<any[]>();
            } catch (e) {
              console.error(e);
              return [];
            }
          })
        );
        resolve(results);
      }
    );

    const embeddings = await ctx.llm.embeddingsModel.generateEmbeddings([
      body.query,
    ]);

    // Note: just use the id of the last item from the breadcrumb
    const breadCrumb = (body.context?.breadcrumbs || []).pop();
    const derivedFiles = breadCrumb?.id
      ? await ctx.repo.files.fetchDirectChildren({
          parentId: breadCrumb.id,
        })
      : [];

    const subDirectories = breadCrumb?.id
      ? [
          breadCrumb?.id,
          ...(
            await ctx.repo.files.listAllSubDirectories({
              parentId: breadCrumb?.id,
            })
          ).map((dir) => dir.id),
        ]
      : undefined;

    const fileIds = [
      ...derivedFiles.map((f) => f.id),
      ...(breadCrumb?.id ? [breadCrumb?.id] : []),
    ];
    const searchResults = await ctx.repo.embeddings.search({
      embeddings: embeddings[0],
      directories: subDirectories,
      // Pass id and parent id as fileIds in case the context is
      // a file; need to pass parent id since for non plain text files like
      // pdf, fileId will be id of the plain text file, but user can only
      // see orignal file id and hence, original file id will be the context
      fileIds: fileIds.length > 0 ? fileIds : undefined,
      limit: 20,
    });

    const filteredSearchResult = searchResults.filter((result) => {
      return result.score > 0.1;
    });

    const chunksByFile = groupBy(filteredSearchResult, "fileId");
    const uniqueFileIds = uniq(Object.keys(chunksByFile));

    const files =
      uniqueFileIds.length > 0
        ? await ctx.repo.files.fetchFileContent(uniqueFileIds)
        : [];

    const orignalFilesById: Record<string, File> =
      files.length > 0
        ? keyBy(
            await ctx.repo.files
              .fetchByIds(files.map((f) => f.parentId!))
              .then((files) => files.filter((f) => !f.isDirectory)),
            "id"
          )
        : {};

    const fileById = keyBy(
      files.map((file) => {
        const content = file.file?.content;
        return {
          id: file.id,
          name: file.name,
          parentId: file.parentId,
          content,
        };
      }),
      "id"
    );

    const fileChunks = Object.entries(chunksByFile)
      // skip the files not found in filesById
      // this happens if the user doesn't have permission to see
      // matching file
      // TODO: when searching, only search files that the user has
      // access to
      .filter(([fileId, _]) => fileById[fileId])
      .map(([fileId, chunks]) => {
        const file = fileById[fileId];
        // originalFile might be undefined for markdown, txt, etc files. so,
        // use id of the file itself
        const originalFile = orignalFilesById[file.parentId!] || file;
        return {
          id: file.id,
          name: file.name,
          originalFile: {
            id: originalFile.id,
            name: originalFile.name,
          },
          chunks: chunks
            .map((chunk) => {
              const { metadata } = chunk;
              return {
                id: chunk.id,
                score: chunk.score,
                start: metadata.start,
                end: metadata.end,
                content: file.content!.substring(metadata.start, metadata.end),
              };
            })
            .sort((c1, c2) => c1.score - c2.score),
        };
      });

    return {
      files: [
        ...fileChunks,
        ...sharedFilesSearchResults.flatMap((res) => res.files),
      ],
      tools: [],
    };
  });

const listUserAccess = p.query(async ({ ctx, searchParams }) => {
  const entity = searchParams.entity;
  if (!entity) {
    return [];
  }
  const acls = await ky
    .get(
      `${ctx.workspaceHost}/api/acls?appId=${ctx.app.id}&appTemplateId=${ctx.app.template.id}`
    )
    .json<any[]>();
  const entityAcls = acls.filter((acl) => {
    return (
      (ctx.app.ownerId == ctx.user.id || acl.userId == ctx.user.id) &&
      Boolean(
        acl.metadata.entities?.find((aclEntity: any) =>
          aclEntity.id.startsWith(entity)
        )
      )
    );
  });

  // TODO: handle duplicate userId and access
  return uniqBy(entityAcls, (acl) => acl.userId);
});

const shareEntities = p
  .input(
    z.object({
      userId: z.string(),
      access: z.string(),
      entities: z.array(
        z.object({
          id: z.string(),
        })
      ),
    })
  )
  .mutate(async ({ ctx, body, errors }) => {
    // TODO: allow users with admin access to add access
    if (ctx.app.ownerId != ctx.user.id) {
      return errors.forbidden();
    }
    const commands: string[] = [];
    if (body.access == "read-only") {
      commands.push("SELECT");
    } else if (body.access == "full-access") {
      commands.push("INSERT");
      commands.push("UPDATE");
      commands.push("DELETE");
    }

    const filters: any[] = [];
    body.entities.forEach((entity) => {
      if (entity.id) {
        for (const command of commands) {
          filters.push({
            command: "SELECT",
            table: "files",
            condition: `id ILIKE '${entity.id}%'`,
          });
          filters.push({
            command: "SELECT",
            table: "file_embeddings",
            // TODO: narrow down file embeddings access
            condition: `*`,
          });
        }
      }
    });

    await ky
      .post(`${ctx.workspaceHost}/api/acls/add`, {
        json: {
          userId: body.userId,
          accessGroup: body.access,
          app: {
            id: ctx.app.id,
          },
          metadata: {
            filters,
            entities: body.entities,
          },
        },
      })
      .json();
    return { success: true };
  });

const removeUserAccess = p
  .input(
    z.object({
      userId: z.string(),
      entities: z.array(
        z.object({
          id: z.string(),
        })
      ),
    })
  )
  .mutate(async ({ ctx, body, errors }) => {
    // TODO: allow users with admin access to remove access
    if (ctx.app.ownerId != ctx.user.id) {
      return errors.forbidden();
    }

    const acls = await ky
      .get(
        `${ctx.workspaceHost}/api/acls?appId=${ctx.app.id}&appTemplateId=${ctx.app.template.id}&userId=${body.userId}`
      )
      .json<any[]>();

    for (const acl of acls) {
      if (
        acl.userId == body.userId &&
        deepEqual(
          body.entities.map((e) => e.id),
          acl.metadata.entities?.map((e: any) => e.id)
        )
      ) {
        await ky.post(`${ctx.workspaceHost}/api/acls/${acl.id}/archive`).json();
      }
    }
    return { success: true };
  });

export { llmSearch, listUserAccess, removeUserAccess, shareEntities };
