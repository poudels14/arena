import { createRouter, mergedRouter } from "@portal/server-core/router";
import { authenticate } from "./procedure";
import * as account from "./account";
import * as workspaces from "./workspaces";
import * as apps from "./apps";
import * as databases from "./databases";
import * as registry from "./registry";

/**
 * These internal routes are accessible
 * only from the workspace app
 */
const internalRoutes = createRouter({
  prefix: "/api",
  middleware: authenticate.toMiddleware(),
  routes: {
    "/internal/workspaces/add": workspaces.add,
    "/internal/workspaces/list": workspaces.list,
    "/internal/databases/clusters/add": databases.addCluster,
    "/internal/databases/clusters/list": databases.listClusters,
    "/internal/databases/clusters/delete": databases.deleteCluster,
    "/internal/databases/list": databases.list,
    "/internal/apps/add": apps.add,
    "/internal/apps/list": apps.list,
    "/internal/apps/archive": apps.archive,
  },
});

const accountRoutes = createRouter({
  prefix: "/api",
  routes: {
    "/account/signup": account.signup,
    "/account/magicLink": account.sendMagicLink,
    "/account/login/email": account.login,
  },
});

const registryRoutes = createRouter({
  routes: {
    "/registry/upload": registry.put,
    "/registry/*": registry.get,
  },
});

const router = mergedRouter({
  ignoreTrailingSlash: true,
  routers: [registryRoutes, accountRoutes, internalRoutes],
  async middleware({ ctx, next }) {
    // This middleware just logs the error
    try {
      return await next({ ctx });
    } catch (e) {
      console.error(e);
      throw e;
    }
  },
  defaultHandler({ req }) {
    return new Response("404 Not found", {
      status: 404,
    });
  },
});

export { router };
