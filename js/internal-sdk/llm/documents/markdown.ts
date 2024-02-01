import { DocumentSplitter } from "@portal/sdk/llm/splitter";
import { Document, RawDocument } from "./document";

class MarkdownDocument extends Document {
  private content: string;
  constructor(raw: RawDocument) {
    super(raw);
    this.content = this.raw.data.toString("utf-8");
  }

  // Note(sagar): return null since raw and content as same for markdown
  getRaw() {
    return null;
  }

  async getContent() {
    return this.content;
  }

  async getHtml() {
    return null;
  }

  async split(splitter: DocumentSplitter) {
    return await splitter.split({
      type: "text/markdown",
      content: await this.getContent(),
    });
  }
}

export { MarkdownDocument };
