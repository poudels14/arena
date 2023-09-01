import type { DocumentSplitter } from "@arena/llm/splitter";

type RawDocument = {
  filename: string;
  type: string;
  name: string;
  data: typeof Buffer;
};

abstract class Document {
  protected raw: RawDocument;

  constructor(raw: RawDocument) {
    this.raw = raw;
  }

  abstract getContent(): Promise<string>;

  abstract getRaw(): typeof Buffer | null;

  abstract getHtml(): Promise<string | null>;

  abstract split(
    splitter: DocumentSplitter
  ): ReturnType<DocumentSplitter["split"]>;
}

export type { RawDocument };
export { Document };
