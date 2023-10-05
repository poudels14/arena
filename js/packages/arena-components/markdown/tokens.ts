type Token = {
  code: {
    lang: string;
    raw: string;
    text: string;
    type: "code";
  };
  blockquote: { quote: string };
  html: { html: string; block: boolean };
  heading: {
    type: "heading";
    text: string;
    depth: number;
    raw: string;
    slugger: any;
  };
  hr: {};
  list: {
    items: any[];
    loose: boolean;
    ordered: true;
    raw: string;
    start: number;
    type: "list";
  };
  list_item: {
    checked: boolean | undefined;
    loose: boolean;
    raw: string;
    task: boolean;
    text: string;
    tokens: any[];
    type: "list_item";
  };
  checkbox: { checked: string };
  paragraph: {
    raw: string;
    text: string;
    tokens: any;
    type: "paragraph";
  };
  table: { header: string; body: string };
  tablerow: { content: string };
  tablecell: { content: string; flags: object };

  /** The following are inline tokens */
  strong: {
    type: "strong";
    raw: string;
    text: string;
    tokens: any;
  };
  em: {
    type: "em";
    raw: string;
    text: string;
    tokens: any;
  };
  codespan: {
    type: "codespan";
    raw: string;
    text: string;
  };
  br: {};
  del: { text: string };
  link: {
    href: string;
    raw: string;
    text: string;
    tokens: any;
    type: "link";
  };
  image: { href: string; title: string; text: string };
  text: {
    raw: string;
    text: string;
    tokens: any;
    type: "text";
  };
};

export type { Token };
