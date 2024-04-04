import { createRouter, mergedRouter } from "@portal/server-core/router";
import { authenticate } from "./procedure";
import * as account from "./account";
import * as workspaces from "./workspaces";
import * as settings from "./settings";
import * as apps from "./apps";
import * as acls from "./acls";
import * as databases from "./databases";
import * as llm from "./llm";
import * as registry from "./registry";
import * as releases from "./releases";

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
    "/account/findUser": account.findUser,
    "/account/listUsers": account.listUsers,
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
    "/workspaces/:workspaceId/settings": settings.list,
    "/acls/add": acls.addAcl,
    "/acls": acls.listUserAcls,
    "/acls/:id/archive": acls.archiveAcl,
    "/llm/models/add": llm.addCustomModel,
    "/llm/models/update": llm.updateModel,
    "/llm/models/delete": llm.deleteModel,
    "/llm/models": llm.listModels,
  },
});

const registryRoutes = createRouter({
  middleware: authenticate.toMiddleware(),
  routes: {
    "/registry/upload": registry.upload,
    "/registry/*": registry.get,
  },
});

const appReleaseRoutes = createRouter({
  routes: {
    "/desktop/updates/:target/:arch/:currentVersion":
      releases.hasDesktopAppUpdate,
  },
});

const router = mergedRouter({
  ignoreTrailingSlash: true,
  routers: [
    registryRoutes,
    appReleaseRoutes,
    accountRoutes,
    authorizedRoutes,
    internalRoutes,
  ],
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
