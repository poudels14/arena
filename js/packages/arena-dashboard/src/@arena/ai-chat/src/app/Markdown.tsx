import { Marked } from "marked";
import { createResource } from "solid-js";

const Markdown = (props: { content: string }) => {
  const marked = new Marked({
    async: true,
    renderer: {
      heading(text, level) {
        return `<h${level}>${text}</h>`;
      },
      code(text, info, escaped) {
        return `<code class="block my-2 px-4 py-4 rounded bg-gray-800 text-white">${text}</code>`;
      },
      codespan(text) {
        return `<code class="px-1.5 py-0.5 rounded bg-gray-600 text-white">${text}</code>`;
      },
      paragraph(text) {
        return `<p class="py-3">${text}</p>`;
      },
    },
  });

  const [html] = createResource(
    () => props.content,
    (content) => marked.parse(content)
  );

  return (
    <div style={"letter-spacing: 0.2px; word-spacing: 2px"}>
      <div innerHTML={html()}></div>
    </div>
  );
};

export { Markdown };
