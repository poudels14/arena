import multipart from "parse-multipart-data";

const BOUNDARY_REGEX = new RegExp("multipart/form-data; boundary=([\\S]+)");

const parseFormData = async (request: Request) => {
  let contentType = request.headers.get("content-type");
  if (contentType) {
    const [_, boundary] = BOUNDARY_REGEX.exec(contentType) || [];
    if (boundary) {
      const contentLength = parseInt(request.headers.get("content-length")!);
      const buf = await readerToBuffer(
        request.body?.getReader(),
        contentLength
      );
      return buf && multipart.parse(buf, boundary);
    }
  }
};

const readerToBuffer = async (reader: any, contentLength: number) => {
  let buffer;
  let offset = 0;
  let next;
  while ((next = await reader.read()) && !next.done) {
    // Note(sagar): create a buffer after reading the first chunk to
    // avoid having to copy the data if the stream has only one chunk
    if (!buffer) {
      buffer = Buffer.from(next.value, 0, contentLength || 2000);
    } else {
      buffer.fill(next.value, offset, (offset += next.value.length));
    }
  }
  return buffer;
};

export { parseFormData };
