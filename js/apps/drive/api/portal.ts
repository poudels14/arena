import { uniq, keyBy, groupBy } from "lodash-es";
import { Search, searchRequestSchema } from "@portal/workspace-sdk/llm/search";
import { p } from "./procedure";

const llmSearch = p
  .input(searchRequestSchema)
  .mutate(async ({ ctx, body }): Promise<Search.Response> => {
    // TODO: idor check
    const embeddings = await ctx.llm.embeddingsModel.generateEmbeddings([
      body.query,
    ]);

    // Note: just use the id of the last item from the breadcrumb
    const contextDirectory = (body.context?.breadcrumbs || []).pop();
    const searchResults = await ctx.repo.embeddings.search({
      embeddings: embeddings[0],
      directories: contextDirectory?.id
        ? [
            contextDirectory?.id,
            ...(
              await ctx.repo.files.listAllSubDirectories({
                parentId: contextDirectory.id,
              })
            ).map((dir) => dir.id),
          ]
        : undefined,
      limit: 5,
    });

    const filteredSearchResult = searchResults.filter((result) => {
      return result.score > 0.35;
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
