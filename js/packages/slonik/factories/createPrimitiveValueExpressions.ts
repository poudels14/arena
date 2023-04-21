import safeStringify from "@arena/fast-safe-stringify";
import { Logger } from "../Logger";
import { UnexpectedStateError } from "../errors";
import type { PrimitiveValueExpression } from "../types";

const log = Logger.child({
  namespace: "createPrimitiveValueExpressions",
});

export const createPrimitiveValueExpressions = (
  values: readonly unknown[]
): readonly PrimitiveValueExpression[] => {
  const primitiveValueExpressions: any[] = [];

  for (const value of values) {
    if (
      Array.isArray(value) ||
      Buffer.isBuffer(value) ||
      typeof value === "string" ||
      typeof value === "number" ||
      typeof value === "boolean" ||
      // TODO(sagar): add this so that object is sent to Rust as is
      typeof value === "object" ||
      value === null
    ) {
      primitiveValueExpressions.push(value);
    } else {
      log.warn(
        {
          value: JSON.parse(safeStringify(value)),
          values: JSON.parse(safeStringify(values)),
        },
        "unexpected value expression"
      );

      throw new UnexpectedStateError("Unexpected value expression.");
    }
  }

  return primitiveValueExpressions;
};
