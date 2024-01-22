import { Buffer } from "buffer";
import { default as crypto, createHash, randomFillSync } from "./crypto";

Arena.__nodeInternal = {
  ...(Arena.__nodeInternal || {}),
  crypto,
};

const webcrypto = {
  getRandomValues(array) {
    Arena.core.ops.op_node_generate_secret(array);
    return array;
  },
};

export default crypto;
export { createHash, randomFillSync, webcrypto };
