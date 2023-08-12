import multipart from "parse-multipart-data";

const BOUNDARY_REGEX = new RegExp("multipart/form-data; boundary=([\\S]+)");
const WEBKIT_BOUNDARY_FROM_BODY_REGEX = new RegExp("([\\S]+)");

const parseFormData = async (request: Request) => {
  const buffer = await readerToBuffer(request.body?.getReader());

  let _, boundary: undefined | string;
  [_, boundary] =
    BOUNDARY_REGEX.exec(request.headers.get("content-type") || "") || [];
  if (!boundary) {
    [_, boundary] = WEBKIT_BOUNDARY_FROM_BODY_REGEX.exec(
      buffer.subarray(0, 200).toString("utf-8")
    );
    // Note(sagar): if the boundary was parsed from body,
    // need to remove prefix "--" since `multipart.parse` expects
    // boundary without the prefix
    boundary = boundary?.substring(2);
  }

  if (boundary) {
    return multipart.parse(buffer, boundary);
  } else {
    throw new Error("Failed to determine Form Boundary");
  }
};

const readerToBuffer = async (reader: any) => {
  let buffer: Buffer | undefined;
  let offset = 0;
  let next: any;
  while ((next = await reader.read())) {
    if (next.value) {
      // Note(sagar): create a buffer after reading the first chunk to
      // avoid having to copy the data if the stream has only one chunk
      if (!buffer) {
        buffer = Buffer.from(next.value, 0, next.value.length);
        offset += next.value.length;
      } else {
        buffer.fill(next.value, offset, (offset += next.value.length));
      }
    }
    if (next.done) {
      break;
    }
  }
  return buffer!;
};

export { parseFormData };
