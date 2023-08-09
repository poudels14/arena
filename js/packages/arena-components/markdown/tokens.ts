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
    text: string;
    level: number;
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
  strong: { text: string };
  em: { text: string };
  codespan: {
    raw: string;
    text: string;
    type: "codespan";
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
