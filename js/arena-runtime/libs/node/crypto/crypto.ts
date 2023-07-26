// credit: deno
import { randomFillSync } from "./randomFill";

// This is very basic crypto module and may not work properly in all cases
const { ops } = Arena.core;

// TODO(@littledivy): Use Result<T, E> instead of boolean when
// https://bugs.chromium.org/p/v8/issues/detail?id=13600 is fixed.
function unwrapErr(ok: boolean) {
  if (!ok) {
    throw new Error("Context is not initialized");
  }
}

const coerceToBytes = (data: string | BufferSource): Uint8Array => {
  if (data instanceof Uint8Array) {
    return data;
  } else if (typeof data === "string") {
    // This assumes UTF-8, which may not be correct.
    return new TextEncoder().encode(data);
  } else if (ArrayBuffer.isView(data)) {
    return new Uint8Array(data.buffer, data.byteOffset, data.byteLength);
  } else if (data instanceof ArrayBuffer) {
    return new Uint8Array(data);
  } else {
    throw new TypeError("expected data to be string | BufferSource");
  }
};

export class Hash {
  #context: number;

  constructor(algorithm: string | number) {
    if (typeof algorithm === "string") {
      this.#context = ops.op_node_create_hash(algorithm);
      if (this.#context === 0) {
        throw new TypeError(`Unknown hash algorithm: ${algorithm}`);
      }
    } else {
      this.#context = algorithm;
    }
  }

  update(data: string | ArrayBuffer, _encoding?: string): this {
    if (typeof data === "string") {
      unwrapErr(ops.op_node_hash_update_str(this.#context, data));
    } else {
      unwrapErr(ops.op_node_hash_update(this.#context, coerceToBytes(data)));
    }
    return this;
  }

  digest(encoding: string): string {
    if (encoding === "hex") {
      return ops.op_node_hash_digest_hex(this.#context);
    }

    const digest = ops.op_node_hash_digest(this.#context);
    switch (encoding) {
      case "binary":
        return String.fromCharCode(...digest);
      case "base64":
        return encodeToBase64(digest);
      case "base64url":
        return encodeToBase64Url(digest);
      default:
        throw new Error("not implemented");
    }
  }
}

function createHash(algorithm: string) {
  return new Hash(algorithm);
}

export default {
  createHash,
  randomFillSync,
};
export { createHash, randomFillSync };
