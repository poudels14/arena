import { createMemo } from "solid-js";
import { HiOutlineClipboard } from "solid-icons/hi";
import { Marked } from "marked";
import { Markdown } from "@portal/solid-ui/markdown";

import hljs from "highlight.js/lib/core";
import "highlight.js/styles/atom-one-dark.css";
import jsGrammar from "highlight.js/lib/languages/javascript";
import cssGrammar from "highlight.js/lib/languages/css";
import xmlGrammar from "highlight.js/lib/languages/xml";
import pythonGrammar from "highlight.js/lib/languages/python";
import rustGrammar from "highlight.js/lib/languages/rust";

hljs.registerLanguage("javascript", jsGrammar);
hljs.registerLanguage("css", cssGrammar);
hljs.registerLanguage("html", xmlGrammar);
hljs.registerLanguage("xml", xmlGrammar);
hljs.registerLanguage("python", pythonGrammar);
hljs.registerLanguage("rust", rustGrammar);

const marked = new Marked({});

const MarkdownRenderer = (props: { markdown: string }) => {
  const tokens = createMemo(() => {
    return marked.lexer(props.markdown);
  });

  return (
    <Markdown
      tokens={tokens()}
      renderer={{
        code(props) {
          const highlighted =
            props.lang && hljs.listLanguages().includes(props.lang);
          return (
            <div class="my-2 rounded text-white space-y-0">
              <div class="flex py-1 px-2 text-xs rounded-t bg-gray-600">
                <div class="flex-1">{props.lang}</div>
                <div
                  class="flex px-2 text-[0.5rem] cursor-pointer"
                  onClick={() => {
                    window.navigator &&
                      navigator.clipboard.writeText(props.text);
                  }}
                >
                  <HiOutlineClipboard class="py-0.5" size={14} />
                  <div>Copy</div>
                </div>
              </div>
              <code
                class="block px-4 py-4 rounded-b bg-gray-800 whitespace-pre overflow-auto scroll:h-1 thumb:rounded thumb:bg-gray-400"
                innerHTML={
                  highlighted
                    ? hljs.highlight(props.text, {
                        language: props.lang,
                      }).value
                    : props.text
                }
              />
            </div>
          );
        },
      }}
    />
  );
};

export { MarkdownRenderer as Markdown };
