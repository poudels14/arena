// These are API routes for desktop version
// These are kept separate from cloud version since we don't want
// to desktop version to have support for everything

import { createRouter, mergedRouter } from "@portal/server-core/router";
import { Context, authenticate } from "./procedure";
import * as workspaces from "./workspaces";
import * as settings from "./settings";
import * as llm from "./llm";

const authorizedRoutes = createRouter({
  prefix: "/api",
  middleware: authenticate.toMiddleware(),
  routes: {
    "/workspaces/": workspaces.list,
    "/workspaces/:id": workspaces.get,
    "/workspaces/:workspaceId/settings": settings.list,
    "/llm/models/add": llm.addCustomModel,
    "/llm/models/update": llm.updateModel,
    "/llm/models/delete": llm.deleteModel,
    "/llm/models": llm.listModels,
  },
});

const router = mergedRouter<
  Pick<Context, "env" | "dbpool" | "host" | "user" | "repo">
>({
  ignoreTrailingSlash: true,
  routers: [
    // @ts-expect-error
    authorizedRoutes,
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
