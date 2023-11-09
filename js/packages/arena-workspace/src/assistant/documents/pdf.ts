import { PdfPage, convertToHtml } from "@arena/cloud/pdf";
import { extractText } from "@arena/cloud/html";
import { DocumentSplitter } from "@arena/llm/splitter";
import { Document, RawDocument } from "./document";

const PAGE_DELIMETER = "\n\n";
class PdfDocument extends Document {
  private pages: PdfPage[] | undefined;
  // text of each page in the pdf
  private pageTexts: string[] | undefined;
  private content: string | undefined;
  private html: string | undefined;

  constructor(raw: RawDocument) {
    super(raw);
  }

  getRaw() {
    return this.raw.data;
  }

  async getContent() {
    if (!this.content) {
      await this.initPages();
      this.content = this.pageTexts!.join(PAGE_DELIMETER);
    }
    return this.content!;
  }

  async getHtml() {
    if (!this.html) {
      await this.initPages();
      const pagesStr = this.pages!.map((p) => p.html).join("");
      this.html = `<div class="pdf-pages">${pagesStr}</div>`;
    }
    return this.html;
  }

  async split(splitter: DocumentSplitter) {
    await this.initPages();
    return (
      await Promise.all(
        this.pageTexts!.map(async (page, pageIdx) => {
          const chunks = await splitter.split({
            type: "text/plain",
            content: page,
          });

          return chunks.map(({ content, position }) => {
            // add the length of all previous pages + page delimeters
            const offset =
              [...Array(pageIdx)].reduce(
                (agg, _, i) => agg + this.pageTexts![i].length,
                0
              ) +
              pageIdx * PAGE_DELIMETER.length;
            return {
              content,
              position: {
                // adjust the offset to global content instead of page content
                start: position.start + offset,
                end: position.end + offset,
              },
              metadata: {
                page: pageIdx + 1,
                offset: position,
              },
            };
          });
        })
      )
    ).flatMap((m) => m);
  }

  private async initPages() {
    if (!this.pages) {
      this.pages = await convertToHtml(this.raw.data);
      this.pageTexts = await Promise.all(
        this.pages.map(async (page) => {
          const texts = await extractText(page.html);
          return texts.join(" ");
        })
      );
    }
  }
}

export { PdfDocument };
