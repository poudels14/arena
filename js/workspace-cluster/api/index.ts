import { createRouter, mergedRouter } from "@portal/server-core/router";
import { p } from "./procedure";

const system = createRouter({
  routes: {
    "/healthy": p.query(() => {
      return "Ok";
    }),
  },
});

const router = mergedRouter({
  prefix: "/api",
  ignoreTrailingSlash: true,
  routers: [system],
  async middleware({ ctx, next }) {
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
