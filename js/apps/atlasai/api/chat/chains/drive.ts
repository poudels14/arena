import {
  BaseCallbackConfig,
  Callbacks,
} from "@langchain/core/callbacks/manager";
import { DocumentInterface } from "@langchain/core/dist/documents/document";
import { BaseRetriever } from "@langchain/core/retrievers";
import { uniqueId } from "@portal/sdk/utils/uniqueId";
import { Search } from "@portal/workspace-sdk/llm/search";
import { dset } from "dset";
import { klona } from "klona";
import ky from "ky";
import { Context } from "~/api/procedure";

class AtalasDriveSearch extends BaseRetriever {
  workspaceHost: string;
  repo: Context["repo"];
  threadId: string;
  context: { app: { id: string }; breadcrumbs: { id: string }[] }[];
  constructor(
    workspaceHost: string,
    repo: Context["repo"],
    threadId: string,
    context: AtalasDriveSearch["context"]
  ) {
    super();
    this.workspaceHost = workspaceHost;
    this.repo = repo;
    this.threadId = threadId;
    this.context = context;
  }

  async getRelevantDocuments(
    query: string,
    config?: BaseCallbackConfig | Callbacks | undefined
  ): Promise<DocumentInterface<Record<string, any>>[]> {
    let searchResults: (Search.Response & { app: { id: string } })[] = [];
    if (this.context.length > 0) {
      const results = await Promise.all(
        this.context.map(async (context) => {
          const { app, breadcrumbs } = context;
          // TODO: error handling
          const activeContextSearchResult = await ky
            .post(
              new URL(
                `/w/apps/${app.id}/api/portal/llm/search?allApps=true`,
                this.workspaceHost
              ).href,
              {
                json: {
                  query,
                  context: {
                    breadcrumbs,
                  },
                },
              }
            )
            .json<Search.Response>();

          if (
            activeContextSearchResult.files.length > 0 ||
            activeContextSearchResult.tools.length > 0
          ) {
            searchResults.push({
              app: {
                id: app.id,
              },
              files: activeContextSearchResult.files.map((file) => {
                return {
                  ...file,
                  // only take top 4 chunks for each file for now
                  chunks: file.chunks
                    .sort((a, b) => a.score - b.score)
                    .slice(0, 4),
                };
              }),
              tools: activeContextSearchResult.tools,
            });
          }
        })
      );
    }

    if (searchResults.length > 0) {
      const clonedSearchResults = klona(searchResults);
      clonedSearchResults.forEach((result) => {
        result.files.forEach((file) => {
          file.chunks.forEach((chunk) => {
            // clear the chunk content to avoid storing duplicate data
            dset(chunk, "content", undefined);
          });
        });
      });

      await this.repo.chatMessages.insert({
        id: uniqueId(21),
        message: {
          content: "",
        },
        threadId: this.threadId,
        role: "system",
        userId: null,
        createdAt: new Date(),
        metadata: {
          searchResults: clonedSearchResults,
        },
        parentId: null,
      });
    }

    const finalResults = searchResults
      .flatMap((result) => result.files)
      .map((file) => {
        return {
          pageContent: file.chunks.map((c) => c.content).join("\n\n...\n\n"),
          metadata: {
            filename: file.name,
          },
        };
      });

    return finalResults;
  }

  get lc_namespace() {
    return ["atlasai"];
  }
}

export { AtalasDriveSearch };
