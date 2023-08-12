import { Parent as ParentNode } from "mdast";

type SplitterOptions = {
  tokens: {
    inputIds: number[];
    offsetMapping: number[][];
  };
  maxTokenLength: number;
  textSplitOverlap: number;
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
  windowTerminationNodes: string[];
};

const visitChildren = function (visitor: any) {
  return function* (node: ParentNode, options: SplitterOptions) {
    let tokens = {
      ...options.tokens,
    };
    // remove first and last tokens and mappings if they are [CLS] and [SEP]
    let firstToken = tokens.offsetMapping[0];
    let lastToken = tokens.offsetMapping[tokens.offsetMapping.length - 1];
    if (firstToken[0] == firstToken[1] && firstToken[0] == 0) {
      tokens.inputIds.shift();
      tokens.offsetMapping.shift();
    }
    if (lastToken[0] == lastToken[1] && lastToken[0] == 0) {
      tokens.inputIds.pop();
      tokens.offsetMapping.pop();
    }

    let finalOptions = {
      ...options,
      tokens: {
        inputIds: tokens.inputIds,
        offsetMapping: tokens.offsetMapping,
      },
      windowStartTokenIndex: 0,
    };

    if (node.children) {
      let i = 0;
      while (i < node.children.length) {
        let child = node.children[i];

        const gen = visitor(child, i, node, finalOptions);
        let nextIndex;
        for (const [idx, chunk] of gen) {
          yield { value: chunk, done: false };
          nextIndex = idx;
        }

        nextIndex = nextIndex || i + 1;
        i = nextIndex <= i ? i + 1 : nextIndex;
      }
    }
    return { done: true };
  };
};

const getNodeAtIndex = (parent: ParentNode, index: number) => {
  return index < parent.children.length ? parent.children[index] : null;
};

const getStartTokenIndexByOffset = (
  options: SplitterOptions,
  offset: number
) => {
  return options.tokens.offsetMapping.findIndex(
    (o) =>
      o[0] == offset ||
      /**
       * Note(sagar): sometimes, probably due to encoding issues,
       * the llm tokenizer tokens' offset doesn't match with markdown
       * tokens' offset. In that case, just use the first offset greater
       * than the one that we are looking for
       */
      o[0] > offset
  );
};

/**
 * This splitter takes in markdown, tokens and token mappings and
 * splits the markdown in a best way possible.
 *  - It combines consecutive sibling nodes as long as the combined
 *    token length doesn't exceed maxTokenLength
 *  - If the token length for a particular node exceeds maxTokenLength,
 *    it splits node at a children level. If a single TEXT node exceeds
 *    the maxTokenLength, it will split the text at the position of "DOT"
 *    token. If the dot token isn't found in a chunk of
 *    length <= maxTokenLength, it splits at the maxTokenLength.
 */
// TODO(sagar): I think this is generic enough to be used even for HTML
// TODO(sagar): write tests coz this is janky af
const splitMarkdownNodes = visitChildren(function* (
  node: ParentNode,
  index: number,
  parent: ParentNode,
  options: SplitterOptions
) {
  const {
    start: nodeStart,
    end: nodeEnd,
    // overlap is set when splitting the text node
    // @ts-expect-error
    overlap,
  } = node.position!;
  const { tokens, maxTokenLength, textSplitOverlap } = options;
  const windowStartTokenIndex = getStartTokenIndexByOffset(
    options,
    nodeStart.offset!
  );

  if (windowStartTokenIndex == -1) {
    throw new Error(
      "Couldn't find token start index in token offsetMapping\n" +
        "This usually happens when different encoding is used by " +
        "markdown parser and tokenizer"
    );
  }

  const splitChunks: any[] = [];

  let maxTokenIndex = Math.min(
    // Note(sagar): since maxTokenIndex is inclusive when calculating
    // offset, subtract 1
    windowStartTokenIndex + maxTokenLength - 1,
    tokens.inputIds.length - 1
  );

  const maxEndOffset = tokens.offsetMapping[maxTokenIndex][1];
  if (nodeEnd.offset! > maxEndOffset) {
    if (node.type == "text") {
      let cutoffTokenIndex = maxTokenIndex;
      /**
       * Note(sagar): If the text was split where the dot/period token is,
       * then no need to overlap the split chunks.
       */
      let tokensOverlapCount =
        cutoffTokenIndex != maxTokenIndex ? 0 : textSplitOverlap;

      const cutoffOffset = Math.min(tokens.offsetMapping[cutoffTokenIndex][1]);

      splitChunks.push({
        type: "text",
        // @ts-expect-error
        value: node.value.substring(0, cutoffOffset - nodeStart.offset!),
        position: {
          start: node.position?.start,
          end: {
            ...node.position?.end,
            offset: cutoffOffset,
          },
        },
      });

      if (cutoffTokenIndex < tokens.offsetMapping.length - 1) {
        let nextChunkStartTokenIndex =
          cutoffTokenIndex + 1 - tokensOverlapCount;

        // If the next chunk's token length is less than max token length and
        // it is the last chunk of the node, make it overlap with previous chunk
        // so that it has max token length. This is to make sure that last chunks
        // aren't just a few tokens long
        let overlappedTokenCount;
        if (
          nextChunkStartTokenIndex + maxTokenLength >
          tokens.inputIds.length
        ) {
          let startIdx = tokens.inputIds.length - maxTokenLength;
          overlappedTokenCount = nextChunkStartTokenIndex - startIdx;
          nextChunkStartTokenIndex = startIdx;
        }

        let nextChunkStart =
          tokens.offsetMapping[nextChunkStartTokenIndex][0] - nodeStart.offset!;

        parent.children.splice(
          index,
          1,
          { type: "text", value: "<CHUNKED>" },
          {
            type: "text",
            // @ts-expect-error
            value: node.value.substring(nextChunkStart),
            position: {
              start: {
                line: nodeStart.line,
                column: nodeStart.column,
                offset: nodeStart.offset! + nextChunkStart,
              },
              end: nodeEnd,
              // @ts-expect-error
              overlap: {
                tokenLength: overlappedTokenCount,
              },
            },
          }
        );
      }
    } else {
      for (const v of splitMarkdownNodes(node, options)) {
        yield [index + splitChunks.length, v.value];
      }
    }
  } else {
    let currNode;
    let currNodeEndOffset;
    let nodeIndex = index;

    // TODO(sagar): filter which nodes are allowed to be merged when splitting
    // For example, doesn't make a lot of sense for list nodes to merge with
    // next node
    while (
      (currNode = getNodeAtIndex(parent, nodeIndex)) &&
      // Note: only check against the termination node if the chunking window
      // already has at least one node
      (nodeIndex == index ||
        !options.windowTerminationNodes.includes(currNode.type)) &&
      // @ts-expect-error
      (currNodeEndOffset = currNode?.position.end.offset) &&
      currNodeEndOffset <= maxEndOffset
    ) {
      splitChunks.push(currNode);
      nodeIndex += 1;
    }
  }

  const offsets = splitChunks.reduce(
    (agg, chunk) => {
      agg.start = Math.min(chunk.position.start.offset, agg.start);
      agg.end = Math.max(chunk.position.end.offset, agg.end);
      return agg;
    },
    { start: Infinity, end: 0 }
  );

  if (splitChunks.length > 0) {
    yield [
      index + splitChunks.length,
      {
        position: {
          start: offsets.start,
          end: offsets.end,
        },
        tokenOverlap: overlap?.tokenLength || 0,
      },
    ];
  }
});

export { splitMarkdownNodes };
