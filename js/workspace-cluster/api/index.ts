import { createRouter } from "@portal/server-core/router";
import * as account from "./account";

const router = createRouter({
  prefix: "/api",
  ignoreTrailingSlash: true,
  async middleware({ ctx, next }) {
    try {
      return await next({ ctx });
    } catch (e) {
      console.error(e);
      throw e;
    }
  },
  routes: {
    "/account/signup": account.signup,
    "/account/magicLink": account.sendMagicLink,
    "/account/login/email": account.login,
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
