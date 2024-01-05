import { procedure } from "@portal/server-core/router";
import { Pool } from "@arena/runtime/postgres";
import { Repo } from "./repo";
import { Env } from "./env";

type Context = {
  host: string;
  user: any;
  dbpool: Pool;
  repo: Repo;
  env: Env;
};

const p = procedure<Context>();

export { p };
export type { Context };
