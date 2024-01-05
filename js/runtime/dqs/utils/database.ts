import { VectorDatabase } from "../../cloud/vectordb";
import { Flags, Client } from "@arena/runtime/sqlite";
import { DatabaseConfig } from "@portal/sdk/db";
import path from "path";

const { ops } = Arena.core;
const createDbClient = async (options: {
  type: DatabaseConfig["type"];
  name: string;
  baseDir: string;
}) => {
  // @ts-expect-error
  const dbPath = path.join(ops.op_arena_get_base_dir(), options.baseDir);
  switch (options.type) {
    case "sqlite":
      return new Client({
        path: path.join(dbPath, `${options.name}/db.sqlite`),
        flags: Flags.SQLITE_OPEN_CREATE | Flags.SQLITE_OPEN_READ_WRITE,
      });
    case "arena-vectordb":
      return await VectorDatabase.open(path.join(dbPath, options.name));
    default:
      throw new Error("Unsupported database type: " + options.type);
  }
};

export { createDbClient };
