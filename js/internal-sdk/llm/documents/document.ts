import type { DocumentSplitter } from "@portal/sdk/llm/splitter";

type RawDocument = {
  data: Buffer;
};

abstract class Document {
  protected raw: RawDocument;

  constructor(raw: RawDocument) {
    this.raw = raw;
  }

  // Return null if text extraction isnt needed, like for plain text files etc
  abstract getExtractedText(): Promise<string | null>;

  abstract getRaw(): Buffer;

  abstract getHtml(): Promise<string | null>;

  abstract split(
    splitter: DocumentSplitter
  ): ReturnType<DocumentSplitter["split"]>;
}

export type { RawDocument };
export { Document };
