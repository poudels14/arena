import type { DocumentSplitter } from "@portal/sdk/llm/splitter";

type RawDocument = {
  data: Buffer;
};

abstract class Document {
  protected raw: RawDocument;

  constructor(raw: RawDocument) {
    this.raw = raw;
  }

  abstract getContent(): Promise<string>;

  abstract getRaw(): Buffer | null;

  abstract getHtml(): Promise<string | null>;

  abstract split(
    splitter: DocumentSplitter
  ): ReturnType<DocumentSplitter["split"]>;
}

export type { RawDocument };
export { Document };
