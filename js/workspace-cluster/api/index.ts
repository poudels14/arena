import { createRouter, mergedRouter } from "@portal/server-core/router";
import * as account from "./account";
import * as workspaces from "./workspaces";
import * as databases from "./databases";
import { authenticate } from "./procedure";

/**
 * These registry routes are internal routes that are accessible
 * only from the workspace app
 */
const registryRoutes = createRouter({
  middleware: authenticate.toMiddleware(),
  routes: {
    "/registry/workspaces/add": workspaces.add,
    "/registry/workspaces/list": workspaces.list,
    "/registry/databases/clusters/add": databases.addCluster,
    "/registry/databases/clusters/list": databases.listClusters,
    "/registry/databases/clusters/delete": databases.deleteCluster,
    "/registry/databases/list": databases.list,
  },
});

const accountRoutes = createRouter({
  routes: {
    "/account/signup": account.signup,
    "/account/magicLink": account.sendMagicLink,
    "/account/login/email": account.login,
  },
});

const router = mergedRouter({
  ignoreTrailingSlash: true,
  prefix: "/api",
  routers: [accountRoutes, registryRoutes],
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
    const url = new URL(req.url);
    if (url.pathname.startsWith("/api/")) {
      return new Response("404 Not found", {
        status: 404,
      });
    }
  },
});

export { router };
