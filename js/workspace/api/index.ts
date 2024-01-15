import { mergedRouter } from "@portal/server-core/router";

const router = mergedRouter({
  prefix: "/api",
  ignoreTrailingSlash: true,
  routers: [],
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
