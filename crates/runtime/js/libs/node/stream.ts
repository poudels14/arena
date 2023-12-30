import { Readable, PassThrough, pipeline, Transform } from "readable-stream";

const stream = { Readable, PassThrough, pipeline, Transform };

Arena.__nodeInternal = {
  ...(Arena.__nodeInternal || {}),
  stream,
};

export default stream;
export { Readable, PassThrough, pipeline, Transform };
