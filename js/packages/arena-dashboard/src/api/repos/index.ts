import { Client } from "@arena/postgres";
import { createRepo as createAppRepo } from "./app";
import { createRepo as createWidgetsRepo } from "./widget";

const createRepo = (client: Client) => {
  const context = {
    client,
  };
  return Object.assign(
    {},
    { apps: createAppRepo(context) },
    { widgets: createWidgetsRepo(context) }
  );
};

export { createRepo };
