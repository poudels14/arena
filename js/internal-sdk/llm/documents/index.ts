import { RawDocument } from "./document";
import { MarkdownDocument } from "./markdown";
import { PdfDocument } from "./pdf";

enum ContentType {
  TEXT = "text/plain",
  MARKDOWN = "text/markdown",
  PDF = "application/pdf",
}

const createDocument = (contentType: ContentType, document: RawDocument) => {
  switch (contentType) {
    case ContentType.MARKDOWN:
    case ContentType.TEXT:
      return new MarkdownDocument(document);
    case ContentType.PDF:
      return new PdfDocument(document);
    default:
      throw new Error("Unsupported document type");
  }
};

export { ContentType, createDocument };
