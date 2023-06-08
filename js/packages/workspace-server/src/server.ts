import type { Handler } from "@arena/core/server";
import { serve as serveHttp } from "@arena/runtime/server";
import { createFileServer } from "./fileserver";

const serve = async (
  handler: Handler,
  options: { serveFiles?: boolean } = {}
) => {
  // Note(sagar): since this is running in server, set SSR = true
  Arena.env.SSR = true;
  process.env.SSR = true;

  // TODO(sagar): we need to store logs from Arena and logs from queries
  // separately
  console.log("[Arena.Workspace.serve]: Listening to connections...");

  const fileServer = options.serveFiles ? createFileServer() : null;
  await serveHttp({
    async fetch(req) {
      let event = {
        request: req,
        env: Arena.env,
      };

      if (fileServer) {
        let file = await fileServer.execute(event);
        if (file.status !== 404) {
          return file;
        }
      }
      return await handler.execute(event);
    },
  });
};

export { serve };
