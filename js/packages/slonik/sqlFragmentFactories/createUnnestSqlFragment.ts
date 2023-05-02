import { InvalidInputError } from "../errors";
import type {
  SqlFragment,
  UnnestSqlToken,
  PrimitiveValueExpression,
} from "../types";
import {
  countArrayDimensions,
  escapeIdentifier,
  isPrimitiveValueExpression,
  stripArrayNotation,
} from "../utilities";

export const createUnnestSqlFragment = (
  token: UnnestSqlToken,
  greatestParameterPosition: number
): SqlFragment => {
  const { columnTypes } = token;

  const values = [];

  const unnestBindings: PrimitiveValueExpression[][] = [];
  const unnestSqlTokens: string[] = [];

  let columnIndex = 0;

  let placeholderIndex = greatestParameterPosition;

  while (columnIndex < columnTypes.length) {
    let columnType = columnTypes[columnIndex];
    let columnTypeIsIdentifier = typeof columnType !== "string";

    if (typeof columnType !== "string") {
      columnTypeIsIdentifier = true;
      columnType = columnType
        .map((identifierName) => {
          if (typeof identifierName !== "string") {
            throw new InvalidInputError(
              "sql.unnest column identifier name array member type must be a string."
            );
          }

          return escapeIdentifier(identifierName);
        })
        .join(".");
    }

    unnestSqlTokens.push(
      "$" +
        String(++placeholderIndex) +
        "::" +
        (columnTypeIsIdentifier
          ? stripArrayNotation(columnType)
          : escapeIdentifier(stripArrayNotation(columnType))) +
        "[]".repeat(countArrayDimensions(columnType) + 1)
    );

    unnestBindings[columnIndex] = [];

    columnIndex++;
  }

  let lastTupleSize;

  for (const tupleValues of token.tuples) {
    if (
      typeof lastTupleSize === "number" &&
      lastTupleSize !== tupleValues.length
    ) {
      throw new Error(
        "Each tuple in a list of tuples must have an equal number of members."
      );
    }

    if (tupleValues.length !== columnTypes.length) {
      throw new Error("Column types length must match tuple member length.");
    }

    lastTupleSize = tupleValues.length;

    let tupleColumnIndex = 0;

    for (const tupleValue of tupleValues) {
      if (
        !Array.isArray(tupleValue) &&
        !isPrimitiveValueExpression(tupleValue) &&
        !Buffer.isBuffer(tupleValue)
      ) {
        throw new InvalidInputError(
          "Invalid unnest tuple member type. Must be a primitive value expression."
        );
      }

      const tupleBindings = unnestBindings[tupleColumnIndex++];

      if (!tupleBindings) {
        throw new Error("test");
      }

      tupleBindings.push(tupleValue);
    }
  }

  values.push(...unnestBindings);

  const sql = "unnest(" + unnestSqlTokens.join(", ") + ")";

  return {
    sql,
    values,
  };
};
