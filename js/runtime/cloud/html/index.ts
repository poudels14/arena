const { opAsync } = Arena.core;

const extractText = async (htmlContent: String): Promise<string[]> => {
  return await opAsync("op_cloud_html_extract_text", htmlContent, {
    ignoreTags: ["style", "link", "script", "head", "meta"],
    skipWhitespaces: true,
  });
};

export { extractText };
