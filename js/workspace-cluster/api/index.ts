import { createRouter, mergedRouter } from "@portal/server-core/router";
import { authenticate, parseUserFromHeaders } from "./procedure";
import * as account from "./account";
import * as workspaces from "./workspaces";
import * as apps from "./apps";
import * as acls from "./acls";
import * as databases from "./databases";
import * as llm from "./llm";
import * as registry from "./registry";

/**
 * These internal routes are accessible
 * only from the workspace app
 */
const internalRoutes = createRouter({
  // Use `/internal/api` since `/api` is pubic
  prefix: "/internal/api",
  // TODO: only allow internal system to access these routes
  middleware: authenticate.toMiddleware(),
  routes: {
    "/workspaces/add": workspaces.add,
    "/databases/clusters/add": databases.addCluster,
    "/databases/clusters/list": databases.listClusters,
    "/databases/clusters/delete": databases.deleteCluster,
    "/databases/list": databases.list,
    "/apps/add": apps.add,
    "/apps": apps.list,
    "/apps/archive": apps.archive,
  },
});

const accountRoutes = createRouter({
  prefix: "/api",
  routes: {
    "/account/signup": account.signup,
    "/account/login/magic/send": account.sendMagicLink,
    "/account/login/magic": account.magicLinkLogin,
  },
});

const authorizedRoutes = createRouter({
  prefix: "/api",
  middleware: authenticate.toMiddleware(),
  routes: {
    "/workspaces/": workspaces.list,
    "/workspaces/:id": workspaces.get,
    "/acls/add": acls.addAcl,
    "/acls": acls.listAcls,
    "/acls/:id/archive": acls.archiveAcl,
    "/llm": llm.list,
  },
});

const registryRoutes = createRouter({
  middleware: parseUserFromHeaders.toMiddleware(),
  routes: {
    "/registry/upload": registry.upload,
    "/registry/*": registry.get,
  },
});

const router = mergedRouter({
  ignoreTrailingSlash: true,
  routers: [registryRoutes, accountRoutes, authorizedRoutes, internalRoutes],
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
