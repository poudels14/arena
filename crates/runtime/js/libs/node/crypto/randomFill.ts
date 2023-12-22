import { Buffer } from "buffer";
const { ops } = Arena.core;

export const kMaxUint32 = 4294967295;
const kBufferMaxLength = 0x7fffffff;
function assertOffset(offset: number, length: number) {
  if (offset > kMaxUint32 || offset < 0) {
    throw new TypeError("offset must be a uint32");
  }

  if (offset > kBufferMaxLength || offset > length) {
    throw new RangeError("offset out of range");
  }
}

function assertSize(size: number, offset: number, length: number) {
  if (size > kMaxUint32 || size < 0) {
    throw new TypeError("size must be a uint32");
  }

  if (size + offset > length || size > kBufferMaxLength) {
    throw new RangeError("buffer too small");
  }
}

function randomFillSync(buf: Buffer, offset = 0, size?: number) {
  assertOffset(offset, buf.length);

  if (size === undefined) size = buf.length - offset;

  assertSize(size, offset, buf.length);

  const bytes: Uint8Array = new Uint8Array(Math.floor(size));
  ops.op_node_generate_secret(bytes);
  const bytesBuf: Buffer = Buffer.from(bytes.buffer);
  bytesBuf.copy(buf, offset, 0, size);

  return buf;
}

export { randomFillSync };
