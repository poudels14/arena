import hljs from "./highlight";
import { Markdown } from "@arena/components/markdown";
import { Marked } from "marked";

const SUPPORTED_LANGUAGES = hljs.listLanguages();

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
          const highlighted =
            props.lang && SUPPORTED_LANGUAGES.includes(props.lang);
          return (
            <code
              class="block my-2 px-4 py-4 rounded bg-gray-800 text-white overflow-auto"
              innerHTML={
                highlighted
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
