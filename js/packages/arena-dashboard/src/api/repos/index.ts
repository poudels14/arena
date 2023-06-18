import { Client } from "@arena/runtime/postgres";
import { createRepo as createUserRepo } from "./user";
import { createRepo as createAclRepo } from "./acl";
import { createRepo as createAppRepo } from "./app";
import { createRepo as createWidgetsRepo } from "./widget";
import { createRepo as createResourcesRepo } from "./resources";

const createRepo = (client: Client) => {
  const context = {
    client,
  };
  return {
    acl: createAclRepo(context),
    users: createUserRepo(context),
    apps: createAppRepo(context),
    widgets: createWidgetsRepo(context),
    resources: createResourcesRepo(context),
  };
};

type DbRepo = ReturnType<typeof createRepo>;

export { createRepo };
export type { DbRepo };
