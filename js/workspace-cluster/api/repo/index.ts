import { Client } from "@arena/runtime/postgres";
import { drizzle } from "drizzle-orm/postgres-js";
import { createRepo as createUserRepo } from "./user";
import { createRepo as createWorkspaceRepo } from "./workspace";

type Repo = {
  users: ReturnType<typeof createUserRepo>;
  workspaces: ReturnType<typeof createWorkspaceRepo>;
};

const createRepo = (client: Client) => {
  let pg = drizzle(client);
  return {
    users: createUserRepo(pg),
    workspaces: createWorkspaceRepo(pg),
  };
};

export { createRepo };
export type { Repo };
