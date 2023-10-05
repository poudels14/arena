import { JSX, Match, Switch, createMemo, mapArray } from "solid-js";
import type { Token } from "./tokens";

type TokenRenderer<Props> = (
  props: Props,
  renderer: MarkdownRenderer
) => JSX.Element;
type MarkdownRenderer = {
  [K in keyof Token]: TokenRenderer<Token[K]>;
};

type MarkdownProps = {
  tokens: any;
  renderer?: Partial<MarkdownRenderer>;
};

const Markdown = (props: MarkdownProps) => {
  const renderer: any = Object.assign({}, DEFAULT_RENDERER, props.renderer);
  return renderMarkdown(props, renderer);
};

const renderMarkdown = (props: MarkdownProps, renderer: MarkdownRenderer) => {
  let markdown = createMemo(() => {
    const children = mapArray(
      () => props.tokens,
      (token: any) => {
        // @ts-expect-error
        let r = renderer[token.type];
        if (r) {
          return r(token, renderer);
        } else {
          if (token.type == "space") {
            return;
          }
          throw new Error("Unsupported token type:" + token.type);
        }
      }
    );
    return <>{children}</>;
  });

  return markdown();
};

const DEFAULT_RENDERER: MarkdownProps["renderer"] = {
  code: (props) => {
    return (
      <code class="block my-2 px-4 py-4 rounded bg-gray-800 text-white overflow-auto">
        {props.text}
      </code>
    );
  },
  blockquote: (props) => {
    throw new Error("not implemented");
  },
  html: (props) => {
    throw new Error("not implemented");
  },
  heading: (props) => {
    if (props.depth == 1) {
      return <h1>{props.text}</h1>;
    } else if (props.depth == 2) {
      return <h2>{props.text}</h2>;
    } else if (props.depth == 3) {
      return <h3>{props.text}</h3>;
    } else if (props.depth == 4) {
      return <h4>{props.text}</h4>;
    }
  },
  hr: (props) => {
    return <hr />;
  },
  list: (props, renderer) => {
    return (
      <Switch>
        <Match when={props.ordered}>
          <ol class="my-4 ml-4 space-y-3 list-decimal" start={props.start}>
            {/* TODO(sagar): pass in custom renderer */}
            <RenderTokens
              tokens={props.items}
              fallback={null}
              renderer={renderer}
            />
          </ol>
        </Match>
        <Match when={true}>
          <ul class="my-4 space-y-3">
            <RenderTokens
              tokens={props.items}
              fallback={null}
              renderer={renderer}
            />
          </ul>
        </Match>
      </Switch>
    );
  },
  list_item: (props, renderer) => {
    return (
      <li>
        <RenderTokens
          tokens={props.tokens}
          fallback={props.text}
          renderer={renderer}
        />
      </li>
    );
  },
  checkbox: (props) => {
    throw new Error("not implemented");
  },
  paragraph: (props, renderer) => {
    return (
      <p class="my-4">
        <RenderTokens
          tokens={props.tokens}
          fallback={props.text}
          renderer={renderer}
        />
      </p>
    );
  },
  table: (props) => {
    throw new Error("not implemented");
  },
  tablerow: (props) => {
    throw new Error("not implemented");
  },
  tablecell: (props) => {
    throw new Error("not implemented");
  },

  strong: (props, renderer) => {
    return (
      <strong>
        <RenderTokens
          tokens={props.tokens}
          fallback={props.raw}
          renderer={renderer}
        />
      </strong>
    );
  },
  em: (props, renderer) => {
    return (
      <i>
        <RenderTokens
          tokens={props.tokens}
          fallback={props.raw}
          renderer={renderer}
        />
      </i>
    );
  },
  codespan: (props) => {
    return (
      <code class="px-1.5 py-0.5 rounded bg-gray-200 text-black">
        {props.text}
      </code>
    );
  },
  br: (props) => {
    return <br />;
  },
  del: (props) => {
    throw new Error("not implemented");
  },
  link: (props, renderer) => {
    return (
      <a href={props.href} class="underline">
        <RenderTokens
          tokens={props.tokens}
          fallback={props.text}
          renderer={renderer}
        />
      </a>
    );
  },
  image: (props) => {
    throw new Error("not implemented");
  },
  text: (props, renderer) => {
    return (
      <RenderTokens
        tokens={props.tokens}
        // Note(sagar): render text raw since props.text seems escaped
        fallback={props.raw}
        renderer={renderer}
      />
    );
  },
};

const RenderTokens = (props: {
  tokens: any;
  fallback: JSX.Element;
  renderer: MarkdownRenderer;
}) => {
  return (
    <Switch>
      <Match when={props.tokens}>{renderMarkdown(props, props.renderer)}</Match>
      <Match when={true}>{props.fallback}</Match>
    </Switch>
  );
};

export { Markdown };
