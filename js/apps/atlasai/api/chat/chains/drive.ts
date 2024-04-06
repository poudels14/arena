import {
  BaseCallbackConfig,
  Callbacks,
} from "@langchain/core/callbacks/manager";
import {
  DocumentInterface,
  Document,
} from "@langchain/core/dist/documents/document";
import { BaseRetriever } from "@langchain/core/retrievers";
import { uniqueId } from "@portal/sdk/utils/uniqueId";
import { Search } from "@portal/workspace-sdk/llm/search";
import { dset } from "dset";
import { klona } from "klona";
import ky from "ky";
import dedent from "dedent";
import { Context } from "~/api/procedure";

class AtalasDrive extends BaseRetriever {
  workspaceHost: string;
  repo: Context["repo"];
  threadId: string;
  queryMessageId: string;
  context: {
    app: { id: string };
    breadcrumbs: { id: string; contentType?: string }[];
  }[];
  constructor(
    workspaceHost: string,
    repo: Context["repo"],
    threadId: string,
    queryMessageId: string,
    context: AtalasDrive["context"]
  ) {
    super();
    this.workspaceHost = workspaceHost;
    this.repo = repo;
    this.threadId = threadId;
    this.queryMessageId = queryMessageId;
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
        parentId: this.queryMessageId,
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

  async fetchContextImage() {
    if (this.context.length < 1) {
      return null;
    }
    const { app, breadcrumbs } = this.context[0];
    const contextFile = breadcrumbs[breadcrumbs.length - 1];
    if (
      contextFile &&
      ["image/png", "image/jpeg", "image/jpg"].includes(
        contextFile.contentType!
      )
    ) {
      const file = await ky
        .get(
          new URL(
            `/w/apps/${app.id}/api/portal/document/${contextFile.id}`,
            this.workspaceHost
          ).href
        )
        .json<{
          id: string;
          name: string;
          contentType: string;
          file: {
            content: string;
            metadata: any;
          };
          size: number;
        }>();
      return file;
    }

    return null;
  }

  get lc_namespace() {
    return ["atlasai"];
  }
}

const formatDocumentsAsString = (documents: Document[]) => {
  const pageContents = documents.map((doc) => doc.pageContent).join("\n\n");

  if (documents.length == 0) {
    return "";
  }

  return dedent`Use the following pieces of context to answer the question at the end.
    If you don't know the answer, just say that you don't know, don't try to make up an answer.
    ----------------
    ${pageContents}`;
};

export { AtalasDrive, formatDocumentsAsString };
