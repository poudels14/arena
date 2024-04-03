import { createHash } from "crypto";
import { merge, pick, uniqBy } from "lodash-es";
import { z } from "zod";
import mime from "mime";
import ky from "ky";
import qs from "qs";
import { uniqueId } from "@portal/sdk/utils/uniqueId";
import type { ContentType } from "@portal/internal-sdk/llm/documents";
import { createDocument } from "@portal/internal-sdk/llm/documents";
import { createDocumentSplitter } from "@portal/sdk/llm/splitter";
import { p } from "./procedure";

const addDirectory = p
  .input(
    z.object({
      name: z.string(),
      parentId: z.string().nullable(),
    })
  )
  .mutate(async ({ ctx, body, errors }) => {
    let parent = null;
    if (body.parentId != null) {
      parent = await ctx.repo.files.fetchById(body.parentId);
      if (!parent) {
        return errors.badRequest("Invalid parentId");
      }
    }

    const parentsChildren = await ctx.repo.files.fetchDirectChildren({
      parentId: body.parentId,
    });
    if (parentsChildren.some((child) => child.name == body.name)) {
      return errors.badRequest("Duplicate directory name");
    }

    let directory;
    let success = false;
    // creating a directory in a loop such that, if there's id
    // collision between two directories in a same parent, retry
    // again with different id
    for (let i = 0; i < 10; i++) {
      // if parent is null, create an unique id,
      // else add `-{3}` suffix to the parent
      // this makes it easy to implement ACL
      const id = parent ? `${parent.id}-${uniqueId(3)}` : uniqueId(25);
      directory = {
        id,
        name: body.name,
        description: null,
        isDirectory: true,
        parentId: body.parentId,
        createdBy: ctx.user!.id,
        metadata: {},
        size: 0,
        file: null,
        contentType: null,
        contentHash: null,
        createdAt: new Date(),
      };
      await ctx.repo.files.insert(directory).then(() => (success = true));
      if (success) {
        break;
      }
    }
    return merge(pick(directory, "id", "name", "parentId", "isDirectory"), {
      children: [],
    });
  });

// Returns info of a list of files and directories
const getFiles = p.query(async ({ ctx, searchParams }) => {
  const ids =
    typeof searchParams.id == "string" ? [searchParams.id] : searchParams.id;
  const files = await ctx.repo.files.fetchByIds(ids);
  return files.map((file) => {
    return pick(file, "id", "name", "isDirectory");
  });
});

const listDirectory = p.query(async ({ ctx, params, searchParams, errors }) => {
  const directoryId = params.id ? params.id : null;

  // If listing directory from different app, proxy request
  // this happens for shared files and directories
  if (searchParams.app) {
    const dir = await ky
      .get(
        `${ctx.workspaceHost}/w/apps/${searchParams.app}/api/fs/directory/${
          directoryId || ""
        }`
      )
      .json<any>();
    if (!dir.breadcrumbs) {
      dir.parentId = dir.parentId || "shared";
      dir.breadcrumbs.unshift({
        id: "shared",
        name: "Shared with me",
      });
    }
    return dir;
  }

  const directory = await ctx.repo.files.fetchById(directoryId);
  if (!directory && directoryId != null) {
    return errors.notFound("Directory not found");
  }

  const breadcrumbs = await ctx.repo.files.getBreadcrumb({
    directoryId,
  });

  const children = await ctx.repo.files.fetchDirectChildren({
    parentId: directoryId,
  });

  return merge(
    pick(directory, "id", "name", "parentId", "isDirectory", "createdAt"),
    {
      breadcrumbs,
      children: children.map((child) => {
        return merge(
          pick(child, "id", "name", "parentId", "isDirectory", "createdAt"),
          {
            type: child.isDirectory ? null : mime.getType(child.name),
          }
        );
      }),
    }
  );
});

const listSharedDirectories = p.query(async ({ ctx }) => {
  let acls = [];
  try {
    acls = await ky
      .get(`${ctx.workspaceHost}/api/acls?appTemplateId=${ctx.app.template.id}`)
      .json<any[]>();
  } catch (e) {
    return {
      id: "shared",
      name: "Shared with me",
      breadcrumbs: [
        {
          id: "shared",
          name: "Shared with me",
          parentId: null,
        },
      ],
      children: [],
    };
  }

  const sharedFileIds = acls
    .filter((acl) => {
      return acl.appId != ctx.app!.id;
    })
    .map((acl) => {
      return {
        appId: acl.appId,
        entities: (acl.metadata.entities || []).map((e: any) => e.id),
      };
    })
    .filter((app) => app.entities.length > 0);

  const sharedFiles = await Promise.all(
    sharedFileIds.map(async (app) => {
      const query = qs.stringify(
        { id: app.entities },
        { arrayFormat: "repeat" }
      );
      const filesMetadata = await ky
        .get(`${ctx.workspaceHost}/w/apps/${app.appId}/api/fs/files?${query}`)
        .json<any[]>();
      return filesMetadata.map((file) => {
        return merge(file, { appId: app.appId });
      });
    })
  );

  return {
    id: "shared",
    name: "Shared with me",
    breadcrumbs: [
      {
        id: "shared",
        name: "Shared with me",
        parentId: null,
      },
    ],
    children: uniqBy(
      sharedFiles.flatMap((files) => {
        return files.map((file) => {
          return {
            parentId: "shared",
            ...pick(file, "appId", "id", "name", "isDirectory"),
          };
        });
      }),
      (file) => file.id
    ),
  };
});

const uploadFiles = p.mutate(async ({ req, ctx, errors, form }) => {
  const uploadTime = new Date();
  const formData = await form.multipart(req);
  const parentInput = formData.find((input) => input.name == "parentId");
  if (!parentInput) {
    return errors.badRequest("Missing `parentId` field");
  }

  const parentId = parentInput.data.toString();
  const parentDirectory = await ctx.repo.files.fetchById(
    parentId == "null" ? null : parentId
  );

  if (!parentDirectory && parentId != "null") {
    return errors.notFound("Directory not found");
  }

  // if parent is null, create an unique id,
  // else add `-{6}` suffix to the parent
  // this makes it easy to implement ACL
  // adding suffixing of 7 because file wont have children,
  // so can make this longer than suffix for directory (3)
  const createUniqueIdForChildren = () => {
    return parentDirectory
      ? `${parentDirectory.id}-${uniqueId(7)}`
      : uniqueId(25);
  };

  const newFiles = [];
  const repo = await ctx.repo.transaction();
  try {
    for (const formInput of formData) {
      if (formInput.filename) {
        const contentType = mime.getType(formInput.filename) as ContentType;

        const fileContent = formInput.data.toString("base64");

        var originalFileHash = createHash("sha256");
        originalFileHash.update(fileContent);
        const contentHash = originalFileHash.digest("hex");
        const originalFile = {
          id: createUniqueIdForChildren(),
          name: formInput.filename,
          description: null,
          isDirectory: false,
          parentId: parentDirectory?.id || null,
          createdBy: ctx.user!.id,
          metadata: {},
          size: fileContent.length,
          file: {
            content: fileContent,
          },
          contentType,
          contentHash,
          createdAt: uploadTime,
        };
        await repo.files.insert(originalFile);
        newFiles.push(originalFile);

        // for files that support embeddings, add embeddings
        if (
          [ContentType.TEXT, ContentType.MARKDOWN, ContentType.PDF].includes(
            contentType
          )
        ) {
          // TODO: use workflows for text extraction
          const document = createDocument(contentType, formInput);
          const extractedText = await document.getExtractedText();
          const extractedFile = extractedText
            ? {
                id: originalFile.id + "-text",
                name: formInput.filename,
                description: null,
                isDirectory: false,
                parentId: originalFile.id,
                createdBy: ctx.user!.id,
                metadata: {},
                size: extractedText.length,
                file: {
                  content: Buffer.from(extractedText).toString("base64"),
                },
                contentType: null,
                // use the content hash of the original file so that it's
                // this can be deduped without fetching the parent file
                contentHash,
                createdAt: uploadTime,
              }
            : null;
          if (extractedFile) {
            await repo.files.insert(extractedFile);
          }

          const embeddingFile = extractedFile ? extractedFile : originalFile;
          const documentSplitter = createDocumentSplitter({
            async tokenize(content) {
              return await ctx.llm.embeddingsModel.tokenizeText(content, {
                truncate: false,
              });
            },
            maxTokenLength: 200,
            windowTerminationNodes: ["heading", "table", "code"],
          });

          const documentChunks = await document.split(documentSplitter);
          const embeddings = await ctx.llm.embeddingsModel.generateEmbeddings(
            documentChunks.map((chunk) => chunk.content)
          );

          for (let index = 0; index < documentChunks.length; index++) {
            const chunk = documentChunks[index];
            await repo.embeddings.insert({
              id: uniqueId(18),
              embeddings: embeddings[index],
              metadata: {
                start: chunk.position.start,
                end: chunk.position.end,
                chunk: chunk.metadata,
              },
              fileId: embeddingFile.id,
              directoryId: parentDirectory?.id || null,
              createdAt: uploadTime,
            });
          }
        }
      }
    }
    repo.commit();
  } finally {
    await repo.release();
  }
  return {
    files: newFiles.map((file) =>
      pick(file, "id", "name", "parentId", "isDirectory")
    ),
  };
});

const deleteFile = p
  .input(
    z.object({
      id: z.string(),
    })
  )
  .mutate(async ({ ctx, body, errors }) => {
    let file = await ctx.repo.files.fetchById(body.id);
    if (!file) {
      return errors.notFound("File not found");
    }
    const deletedFiles = await ctx.repo.files.deleteById(body.id);
    const deletedFileIds = deletedFiles.map((file) => file.id);
    await ctx.repo.embeddings.deleteByFileIds(deletedFileIds);
    return { success: true };
  });

export {
  addDirectory,
  listDirectory,
  listSharedDirectories,
  getFiles,
  uploadFiles,
  deleteFile,
};
