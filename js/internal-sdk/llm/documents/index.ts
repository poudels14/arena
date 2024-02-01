import { RawDocument } from "./document";
import { MarkdownDocument } from "./markdown";
import { PdfDocument } from "./pdf";

type ContentType = "text/markdown" | "application/pdf";

const ContentTypes = {
  MARKDOWN: "text/markdown",
  PDF: "application/pdf",
};

const createDocument = (contentType: ContentType, document: RawDocument) => {
  switch (contentType) {
    case ContentTypes.MARKDOWN:
      return new MarkdownDocument(document);
    case ContentTypes.PDF:
      return new PdfDocument(document);
    default:
      throw new Error("Unsupported document type");
  }
};

export { ContentTypes, createDocument };
export type { ContentType };
