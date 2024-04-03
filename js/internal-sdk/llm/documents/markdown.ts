import { DocumentSplitter } from "@portal/sdk/llm/splitter";
import { Document, RawDocument } from "./document";

class MarkdownDocument extends Document {
  private rawText: string;
  constructor(raw: RawDocument) {
    super(raw);
    this.rawText = this.raw.data.toString("utf-8");
  }

  // Note(sagar): return null since raw and content as same for markdown
  getRaw() {
    return this.raw;
  }

  async getExtractedText() {
    // return null since no text extraction needed
    return null;
  }

  async getHtml() {
    return null;
  }

  async split(splitter: DocumentSplitter) {
    return await splitter.split({
      type: "text/markdown",
      content: this.rawText,
    });
  }
}

export { MarkdownDocument };
