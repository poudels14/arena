import { Match, Switch, createMemo } from "solid-js";
import { HiOutlineClipboard } from "solid-icons/hi";
import { Marked } from "marked";
import { Markdown } from "@portal/solid-ui/markdown";

import hljs from "highlight.js/lib/core";
import "highlight.js/styles/atom-one-dark.css";
import jsonGrammar from "highlight.js/lib/languages/json";
import jsGrammar from "highlight.js/lib/languages/javascript";
import tsGrammar from "highlight.js/lib/languages/typescript";
import cssGrammar from "highlight.js/lib/languages/css";
import xmlGrammar from "highlight.js/lib/languages/xml";
import pythonGrammar from "highlight.js/lib/languages/python";
import rustGrammar from "highlight.js/lib/languages/rust";
import cGrammar from "highlight.js/lib/languages/c";

hljs.registerLanguage("json", jsonGrammar);
hljs.registerLanguage("javascript", jsGrammar);
hljs.registerLanguage("typescript", tsGrammar);
hljs.registerLanguage("css", cssGrammar);
hljs.registerLanguage("html", xmlGrammar);
hljs.registerLanguage("xml", xmlGrammar);
hljs.registerLanguage("python", pythonGrammar);
hljs.registerLanguage("rust", rustGrammar);
hljs.registerLanguage("c", cGrammar);

const marked = new Marked({});

const MarkdownRenderer = (markdownProps: { markdown: string }) => {
  const tokens = createMemo(() => {
    return marked.lexer(markdownProps.markdown);
  });

  return (
    <Markdown
      tokens={tokens()}
      renderer={{
        code(props) {
          const codeContent = createMemo(() => {
            const highlighted =
              props.lang && hljs.listLanguages().includes(props.lang);
            if (highlighted) {
              return hljs.highlight(props.text || "", {
                language: props.lang,
              });
            } else {
              return hljs.highlightAuto(props.text);
            }
          });
          return (
            <Switch>
              <Match when={!props.text}>
                <div class="markdown hidden">{markdownProps.markdown}</div>
              </Match>
              <Match when={props.text}>
                <div class="my-2 rounded text-white space-y-0">
                  <div class="flex py-1 px-2 text-xs rounded-t bg-gray-600">
                    <div class="flex-1">
                      {codeContent().language || props.lang}
                    </div>
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
                  <pre>
                    <code
                      class="block px-4 py-4 text-xs rounded-b bg-gray-800 whitespace-pre overflow-auto scroll:h-1 thumb:rounded thumb:bg-gray-400"
                      ref={(node) => {
                        if (codeContent().value) {
                          node.innerHTML = codeContent().value;
                        } else {
                          node.innerText = props.text;
                        }
                      }}
                    />
                  </pre>
                </div>
              </Match>
            </Switch>
          );
        },
      }}
    />
  );
};

export { MarkdownRenderer as Markdown };
