import { createRouter, mergedRouter } from "@portal/server-core/router";
import * as account from "./account";
import * as workspace from "./workspace";
import { authenticate } from "./procedure";

const protectedRoutes = createRouter({
  middleware: authenticate.toMiddleware(),
  routes: {
    "/workspace/create": workspace.create,
  },
});

const publicRoutes = createRouter({
  routes: {
    "/account/signup": account.signup,
    "/account/magicLink": account.sendMagicLink,
    "/account/login/email": account.login,
  },
});

const router = mergedRouter({
  prefix: "/api",
  ignoreTrailingSlash: true,
  routers: [publicRoutes, protectedRoutes],
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
