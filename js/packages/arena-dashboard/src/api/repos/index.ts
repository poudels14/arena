import { Client } from "@arena/runtime/postgres";
import { createRepo as createWorkspaceRepo } from "./workspace";
import { createRepo as createUserRepo } from "./user";
import { createRepo as createAclRepo } from "./acl";
import { createRepo as createAppRepo } from "./app";
import { createRepo as createWidgetsRepo } from "./widget";
import { createRepo as createResourcesRepo } from "./resources";
import { createRepo as createWorkflowRunsRepo } from "./workflowRuns";

const createRepo = (client: Client) => {
  const context = {
    client,
  };
  return {
    workspaces: createWorkspaceRepo(context),
    users: createUserRepo(context),
    acl: createAclRepo(context),
    apps: createAppRepo(context),
    widgets: createWidgetsRepo(context),
    resources: createResourcesRepo(context),
    workflowRuns: createWorkflowRunsRepo(context),
  };
};

type DbRepo = ReturnType<typeof createRepo>;

export { createRepo };
export type { DbRepo };
