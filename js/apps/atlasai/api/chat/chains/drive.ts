import ky from "ky";
import dedent from "dedent";
import indentString from "indent-string";
import { Context as CortexContext, Runnable } from "@portal/cortex/core";
import { Search } from "@portal/workspace-sdk/llm/search";
import { Context } from "~/api/procedure";

export type SearchResults = (Search.Response & { app: { id: string } })[];
class AtalasDrive extends Runnable {
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

  get namespace(): string {
    return "atlasai.drive.search";
  }

  async run(ctxt: CortexContext, query: string) {
    let searchResults: SearchResults = [];
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
    return searchResults;
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
}

const formatSearchResultsAsString = (result: SearchResults) => {
  const fileContents = result
    .flatMap((result) => result.files)
    .map((file) => {
      const chunks = file.chunks
        .map((c) => {
          return indentString(
            `<section>\n` +
              indentString(
                dedent`<id>${c.id}</id>
              <content>
              ${
                (c.context?.before || "") + c.content + (c.context?.after || "")
              }
              </content>\n`,
                4
              ) +
              `</section>`,
            4
          );
        })
        .join("\n");
      return (
        `<file>\n` +
        indentString(
          dedent`<name>${file.name}</name>
            <sections>\n` +
            chunks +
            dedent`\n</sections>`,
          4
        ) +
        `\n</file>`
      );
    })
    .join("\n\n");

  if (result.length == 0) {
    return "";
  }

  return dedent`Use the following context to answer the user query.
    If you don't know the answer, just say that you don't know, don't try to make up an answer.
    ----------------
    ${fileContents}`;
};

export { AtalasDrive, formatSearchResultsAsString };
