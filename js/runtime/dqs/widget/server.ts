import { serve } from "@arena/runtime/server";
import { router } from "./router";

serve({
  async fetch(req) {
    const res = await router.route(req, {});
    if (res) {
      return res;
    }
    return new Response("Not found", {
      status: 404,
    });
  },
});
