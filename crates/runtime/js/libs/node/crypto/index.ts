import { default as crypto, createHash, randomFillSync } from "./crypto";

Arena.__nodeInternal = {
  ...(Arena.__nodeInternal || {}),
  crypto,
};

export default crypto;
export { createHash, randomFillSync };
