import { Flags, Client } from "@arena/runtime/sqlite";
import path from "path";

const { ops } = Arena.core;
const createDbClient = async (options: { path: string }) => {
  // @ts-expect-error
  const dbPath = path.join(ops.op_apps_get_app_dir(), options.path);
  const client = new Client({
    path: dbPath,
    flags: Flags.SQLITE_OPEN_CREATE | Flags.SQLITE_OPEN_READ_WRITE,
  });

  return client;
};

export { createDbClient };
