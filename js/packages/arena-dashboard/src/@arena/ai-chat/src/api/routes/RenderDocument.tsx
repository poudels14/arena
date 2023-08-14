import hljs from "./highlight";
import { Markdown } from "@arena/components/markdown";
import { Marked } from "marked";

const Document = (props: { content: string }) => {
  const marked = new Marked({});
  const tokens = marked.lexer(props.content);

  // TODO(sagar): maybe just use default marked render with tailwind prose
  // with SSR styles
  return (
    <Markdown
      tokens={tokens}
      renderer={{
        code(props) {
          return (
            <code
              class="block my-2 px-4 py-4 rounded bg-gray-800 text-white overflow-auto"
              innerHTML={
                props.lang && hljs.listLanguages().includes(props.lang)
                  ? hljs.highlight(props.text, {
                      language: props.lang,
                      ignoreIllegals: true,
                    }).value
                  : props.text
              }
            />
          );
        },
      }}
    />
  );
};

export default Document;
