import { createRouter, mergedRouter } from "@portal/server-core/router";
import { p } from "./procedure";
import * as files from "./files";

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
    "/fs/directory": files.listDirectory,
    "/fs/directory/add": files.addDirectory,
    "/fs/upload": files.uploadFiles,
  },
});

const systemAIRoutes = createRouter({
  routes: {
    "/internal/ai/search": p.query(() => {
      // TODO: return LLM embedding search results
      return "Ok";
    }),
  },
});

const router = mergedRouter({
  ignoreTrailingSlash: true,
  routers: [system, protectedRoutes],
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
