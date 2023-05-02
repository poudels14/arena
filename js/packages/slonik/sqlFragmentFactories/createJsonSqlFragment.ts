import safeStringify from "@arena/fast-safe-stringify";
import { isPlainObject } from "is-plain-object";
import { serializeError } from "serialize-error";
import { Logger } from "../Logger";
import { InvalidInputError } from "../errors";
import type { JsonSqlToken, SqlFragment } from "../types";

const log = Logger.child({
  namespace: "createJsonSqlFragment",
});

export const createJsonSqlFragment = (
  token: JsonSqlToken,
  greatestParameterPosition: number
): SqlFragment => {
  let value;

  if (token.value === undefined) {
    throw new InvalidInputError("JSON payload must not be undefined.");
  } else if (token.value === null) {
    value = token.value;

    // @todo Deep check Array.
  } else if (
    !isPlainObject(token.value) &&
    !Array.isArray(token.value) &&
    !["number", "string", "boolean"].includes(typeof token.value)
  ) {
    throw new InvalidInputError(
      "JSON payload must be a primitive value or a plain object."
    );
  } else {
    try {
      value = safeStringify(token.value);
    } catch (error) {
      log.error(
        {
          error: serializeError(error),
        },
        "payload cannot be stringified"
      );

      throw new InvalidInputError("JSON payload cannot be stringified.");
    }

    if (value === undefined) {
      throw new InvalidInputError(
        "JSON payload cannot be stringified. The resulting value is undefined."
      );
    }
  }

  // Do not add `::json` as it will fail if an attempt is made to insert to jsonb-type column.
  return {
    sql: "$" + String(greatestParameterPosition + 1),
    values: [value],
  };
};
