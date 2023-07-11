import { serve } from "@arena/runtime/server";
import { router } from "builtin:///@arena/dqs/router";

serve({
  async fetch(req) {
    const url = new URL(req.url);
    // Note(sagar): add a /_healthy endpoint to check server's ability
    // to serve a request
    if (url.pathname === "/_healthy") {
      return "OK";
    }
    return await router.route(req, {});
  },
});
