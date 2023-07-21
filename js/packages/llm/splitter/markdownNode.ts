import { toMarkdown } from "mdast-util-to-markdown";

type SplitterOptions = {
  tokens: {
    inputIds: number[];
    offsetMapping: number[][];
  };
  maxTokenLength: number;
  specialTokens: {
    dot: number;
  };
  textSplitOverlap: number;
};

const visitChildren = function (visitor) {
  return function* (node, options: SplitterOptions) {
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
          if (chunk.text.trim().length > 0) {
            yield { value: chunk, done: false };
          }
          nextIndex = idx;
        }

        nextIndex = nextIndex || i + 1;
        i = nextIndex <= i ? i + 1 : nextIndex;
      }
    }
    return { done: true };
  };
};

const getNodeAtIndex = (parent, index) => {
  return index < parent.children.length ? parent.children[index] : null;
};

const dotTokenIndex = (startIndex, endIndex, options: SplitterOptions) => {
  let dotIndex = endIndex;
  while (dotIndex > startIndex) {
    if (options.tokens.inputIds[dotIndex] == options.specialTokens.dot) {
      return dotIndex;
    }
    dotIndex -= 1;
  }
  return endIndex;
};

const getChunkTokens = (options: SplitterOptions, startOffset, endOffset) => {
  const { offsetMapping } = options.tokens;
  let startIdx = offsetMapping.findIndex((o) => o[0] == startOffset);
  const endIdx = offsetMapping.findIndex((o) => o[1] == endOffset);
  // Since end index is exclusize for `[].slice` add 1
  return offsetMapping.slice(startIdx, endIdx + 1);
};

const getStartTokenIndexByOffset = (
  options: SplitterOptions,
  offset: number
) => {
  return options.tokens.offsetMapping.findIndex((o) => o[0] == offset);
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
const markdownNodeSplitter = visitChildren(function* (
  node,
  index,
  parent,
  options: SplitterOptions
) {
  const {
    position: {
      start: nodeStart,
      end: nodeEnd,
      // overlap is set when splitting the text node
      overlap,
    },
  } = node;
  const { tokens, maxTokenLength, textSplitOverlap } = options;
  const windowStartTokenIndex = getStartTokenIndexByOffset(
    options,
    nodeStart.offset
  );
  let splitChunkTokens;
  const splitChunks: any[] = [];

  let maxTokenIndex = Math.min(
    // Note(sagar): since maxTokenIndex is inclusive when calculating
    // offset, subtract 1
    windowStartTokenIndex + maxTokenLength - 1,
    tokens.inputIds.length - 1
  );

  const maxEndOffset = tokens.offsetMapping[maxTokenIndex][1];
  if (nodeEnd.offset > maxEndOffset) {
    if (node.type == "text") {
      let cutoffTokenIndex = dotTokenIndex(
        windowStartTokenIndex,
        maxTokenIndex,
        options
      );
      /**
       * Note(sagar): If the text was split where the dot/period token is,
       * then no need to overlap the split chunks.
       */
      let tokensOverlapCount =
        cutoffTokenIndex != maxTokenIndex ? 0 : textSplitOverlap;

      const cutoffOffset = Math.min(tokens.offsetMapping[cutoffTokenIndex][1]);

      splitChunks.push({
        type: "text",
        value: node.value.substring(0, cutoffOffset - nodeStart.offset),
      });
      splitChunkTokens = getChunkTokens(
        options,
        nodeStart.offset,
        cutoffOffset
      );

      if (cutoffTokenIndex < tokens.offsetMapping.length) {
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
          tokens.offsetMapping[nextChunkStartTokenIndex][0] - nodeStart.offset;

        parent.children.splice(
          index,
          1,
          { type: "text", value: "<CHUNKED>" },
          {
            type: "text",
            value: node.value.substring(nextChunkStart),
            position: {
              start: {
                line: nodeStart.line,
                column: nodeStart.column,
                offset: nodeStart.offset + nextChunkStart,
              },
              end: nodeEnd,
              overlap: {
                tokenLength: overlappedTokenCount,
              },
            },
          }
        );
      }
    } else {
      for (const v of markdownNodeSplitter(node, options)) {
        yield [index + splitChunks.length, v.value];
      }
    }
  } else {
    let currNode;
    let lastNode;
    let currNodeEndOffset;
    let nodeIndex = index;

    // TODO(sagar): filter which nodes are allowed to be merged when splitting
    // For example, doesn't make a lot of sense for list nodes to merge with
    // next node
    while (
      (currNode = getNodeAtIndex(parent, nodeIndex)) &&
      (currNodeEndOffset = currNode?.position.end.offset) &&
      currNodeEndOffset <= maxEndOffset
    ) {
      splitChunks.push(currNode);
      nodeIndex += 1;
      lastNode = currNode;
    }

    splitChunkTokens = getChunkTokens(
      options,
      nodeStart.offset,
      lastNode.position.end.offset
    );
  }

  yield [
    index + splitChunks.length,
    {
      text: toMarkdown({
        type: "root",
        children: splitChunks,
      }),
      // TODO(sagar): there seems to be some bug when calculating tokenLength
      // and tokenOverlap
      tokens: splitChunkTokens,
      tokenOverlap: overlap?.tokenLength || 0,
    },
  ];
});

export { markdownNodeSplitter };
