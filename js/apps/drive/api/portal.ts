import { uniq, keyBy, groupBy } from "lodash-es";
import { Search, searchRequestSchema } from "@portal/workspace-sdk/llm/search";
import { p } from "./procedure";
import { P } from "drizzle-orm/db.d-a6fe1b19";

const llmSearch = p
  .input(searchRequestSchema)
  .mutate(async ({ ctx, body }): Promise<Search.Response> => {
    // TODO: idor check
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
      limit: 10,
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

    const orignalFilesById =
      files.length > 0
        ? keyBy(
            await ctx.repo.files.fetchFileNamesByIds(
              files.map((f) => f.parentId!)
            ),
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

    const fileChunks = Object.entries(chunksByFile).map(([fileId, chunks]) => {
      const file = fileById[fileId];
      const originalFile = orignalFilesById[file.parentId!];
      return {
        id: file.id,
        name: file.name,
        originalFile: {
          id: originalFile.id,
          name: originalFile.name,
        },
        chunks: chunks.map((chunk) => {
          const { metadata } = chunk;
          return {
            id: chunk.id,
            score: chunk.score,
            start: metadata.start,
            end: metadata.end,
            content: file.content!.substring(metadata.start, metadata.end),
          };
        }),
      };
    });

    return {
      files: fileChunks,
      tools: [],
    };
  });

export { llmSearch };
