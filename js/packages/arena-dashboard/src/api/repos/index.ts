import { Client } from "@arena/runtime/postgres";
import { createRepo as createAppRepo } from "./app";
import { createRepo as createWidgetsRepo } from "./widget";
import { createRepo as createResourcesRepo } from "./resources";

const createRepo = (client: Client) => {
  const context = {
    client,
  };
  return {
    apps: createAppRepo(context),
    widgets: createWidgetsRepo(context),
    resources: createResourcesRepo(context),
  };
};

export { createRepo };
