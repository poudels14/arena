import safeStringify from "@arena/fast-safe-stringify";
import { Logger } from "./Logger";
import { InvalidInputError } from "./errors";
import {
  ArrayToken,
  BinaryToken,
  IdentifierToken,
  JsonToken,
  ListToken,
  SqlToken,
  UnnestToken,
} from "./tokens";
import type {
  ArraySqlToken,
  BinarySqlToken,
  IdentifierSqlToken,
  JsonSqlToken,
  SqlToken as SqlTokenType,
  ListSqlToken,
  PrimitiveValueExpression,
  QueryResultRow,
  SerializableValue,
  SqlSqlToken,
  SqlTaggedTemplate,
  TypeNameIdentifier,
  UnnestSqlColumn,
  UnnestSqlToken,
  ValueExpression,
} from "./types";
import {
  escapeLiteralValue,
  isPrimitiveValueExpression,
  isSqlToken,
} from "./utilities";
import { createSqlTokenSqlFragment } from "./factories/createSqlTokenSqlFragment";

const log = Logger.child({
  namespace: "sql",
});

const sql: SqlTaggedTemplate = (
  parts: readonly string[],
  ...values: readonly ValueExpression[]
): SqlSqlToken => {
  let rawSql = "";

  const parameterValues: any[] = [];

  let index = 0;

  for (const part of parts) {
    const token = values[index++];

    rawSql += part;

    if (index >= parts.length) {
      continue;
    }

    if (token === undefined) {
      log.debug(
        {
          index,
          parts: JSON.parse(safeStringify(parts)),
          values: JSON.parse(safeStringify(values)),
        },
        "bound values"
      );

      throw new InvalidInputError(
        "SQL tag cannot be bound an undefined value."
      );
    } else if (isPrimitiveValueExpression(token)) {
      rawSql += "$" + String(parameterValues.length + 1);

      parameterValues.push(token);
    } else if (isSqlToken(token)) {
      const sqlFragment = createSqlTokenSqlFragment(
        token,
        parameterValues.length
      );

      rawSql += sqlFragment.sql;
      parameterValues.push(...sqlFragment.values);
    } else {
      log.error(
        {
          constructedSql: rawSql,
          index,
          offendingToken: JSON.parse(safeStringify(token)),
        },
        "unexpected value expression"
      );

      throw new TypeError("Unexpected value expression.");
    }
  }

  const query: SqlTokenType = {
    sql: rawSql,
    type: SqlToken,
    values: parameterValues,
  };

  Object.defineProperty(query, "sql", {
    configurable: false,
    enumerable: true,
    writable: false,
  });

  return query;
};

sql.array = (
  values: readonly PrimitiveValueExpression[],
  memberType: SqlTokenType | TypeNameIdentifier | string
): ArraySqlToken => {
  return {
    memberType,
    type: ArrayToken,
    values,
  };
};

sql.binary = (data: Buffer): BinarySqlToken => {
  return {
    data,
    type: BinaryToken,
  };
};

sql.identifier = (names: readonly string[]): IdentifierSqlToken => {
  return {
    names,
    type: IdentifierToken,
  };
};

sql.json = (value: SerializableValue): JsonSqlToken => {
  return {
    type: JsonToken,
    value,
  };
};

sql.join = (
  members: readonly ValueExpression[],
  glue: SqlSqlToken
): ListSqlToken => {
  return {
    glue,
    members,
    type: ListToken,
  };
};

sql.literalValue = (value: string): SqlSqlToken => {
  return {
    sql: escapeLiteralValue(value),
    type: SqlToken,
    values: [],
  };
};

sql.unnest = (
  tuples: ReadonlyArray<readonly PrimitiveValueExpression[]>,
  columnTypes: readonly UnnestSqlColumn[]
): UnnestSqlToken => {
  return {
    columnTypes,
    tuples,
    type: UnnestToken,
  };
};

export const createSqlTag = <
  T extends QueryResultRow = QueryResultRow
>(): SqlTaggedTemplate<T> => {
  return sql;
};
