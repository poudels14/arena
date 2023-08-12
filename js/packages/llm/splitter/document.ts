import { fromMarkdown } from "mdast-util-from-markdown";
import { splitMarkdownNodes } from "./markdownNode";

namespace Splitter {
  export type Options = {
    tokenize: (
      content: string
    ) => Promise<{ ids: number[]; offsetMapping: number[][] }>;
    maxTokenLength: number;
    textSplitOverlap?: number;
    /**
     * List of node types where the chunk's context window should terminate.
     * For example, if we want to split the following text into two nodes at
     * heading, even if the entire text fits into the content length, pass
     * "heading" as termination node;
     * `
     * # heading1
     * some text
     * # heading 2
     * some text for heading two
     * `
     */
    windowTerminationNodes?: string[];
  };
  export type Document = {
    type: "markdown";
    content: string;
  };

  export type DocumentChunk = {
    content: string;
    position: {
      start: number;
      end: number;
    };
  };
}

const createDocumentSplitter = (options: Splitter.Options) => {
  return async (
    document: Splitter.Document
  ): Promise<Splitter.DocumentChunk[]> => {
    switch (document.type) {
      case "markdown": {
        const tokens = await options.tokenize(document.content);

        const nodes = fromMarkdown(document.content);
        const chunks = splitMarkdownNodes(nodes, {
          tokens: {
            inputIds: tokens.ids,
            offsetMapping: tokens.offsetMapping,
          },
          maxTokenLength: options.maxTokenLength,
          textSplitOverlap: options.textSplitOverlap || 0,
          windowTerminationNodes: options.windowTerminationNodes || [],
        });

        return Array.from(chunks).map((c) => {
          const { position } = c.value;
          const { content } = document;
          return {
            ...c.value,
            content: content.substring(position.start, position.end),
          };
        });
      }
      default:
        throw new Error("Unsupported document type:" + document.type);
    }
  };
};

export { createDocumentSplitter };
export type { Splitter };
