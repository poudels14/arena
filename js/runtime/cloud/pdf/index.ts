declare var Arena;

type ConvertToHtmlOptions = {
  pdfiumPath?: string;
};

type PdfPage = {
  html: string;
};

const { opAsync } = Arena.core;
const convertToHtml = async (
  pdfContent: ArrayBuffer,
  options?: ConvertToHtmlOptions
): Promise<PdfPage[]> => {
  return await opAsync("op_cloud_pdf_to_html", pdfContent, options || {});
};

export type { PdfPage };
export { convertToHtml };
