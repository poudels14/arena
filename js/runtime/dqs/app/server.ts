import { serve } from "@arena/runtime/server";
// @ts-expect-error
import appServer from "@dqs/template/app";

serve({
  async fetch(req) {
    console.log(req);
    if (appServer?.fetch) {
      const res = await appServer.fetch(req);
      if (res instanceof Response) {
        return res;
      }
    }
    return new Response("Not found", {
      status: 404,
    });
  },
});
