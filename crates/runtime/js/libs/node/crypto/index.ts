import { Buffer } from "buffer";
import { default as crypto, createHash, randomFillSync } from "./crypto";

Arena.__nodeInternal = {
  ...(Arena.__nodeInternal || {}),
  crypto,
};

const webcrypto = {
  getRandomValues(array) {
    randomFillSync(Buffer.from(array));
    return array;
  },
};

export default crypto;
export { createHash, randomFillSync, webcrypto };
