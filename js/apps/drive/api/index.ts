import { createRouter, mergedRouter } from "@portal/server-core/router";
import { p } from "./procedure";
import * as files from "./files";
import * as portal from "./portal";

const system = createRouter({
  prefix: "/api",
  routes: {
    "/healthy": p.query(() => {
      return "Ok";
    }),
  },
});

const protectedRoutes = createRouter({
  prefix: "/api",
  routes: {
    "/fs/directory/shared": files.listSharedDirectories,
    "/fs/directory/add": files.addDirectory,
    "/fs/directory/:id?": files.listDirectory,
    "/fs/files": files.getFiles,
    "/fs/upload": files.uploadFiles,
  },
});

const portalRoutes = createRouter({
  routes: {
    "/api/portal/llm/search": portal.llmSearch,
  },
});

const router = mergedRouter({
  ignoreTrailingSlash: true,
  routers: [system, protectedRoutes, portalRoutes],
  middleware: async ({ ctx, next }) => {
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
