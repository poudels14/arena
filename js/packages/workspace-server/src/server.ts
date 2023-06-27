import type { Handler } from "@arena/core/server";
import { serve as arenaServer } from "@arena/runtime/server";
import { createFileServer } from "./fileserver";

// Note(sagar): since this is running in server, set SSR = true
Arena.env.SSR = "true";
process.env.SSR = "true";

const serve = async (
  handler: Handler,
  options: { serveFiles?: boolean } = {}
) => {
  // TODO(sagar): we need to store logs from Arena and logs from queries
  // separately
  console.log("[Arena.Workspace.serve]: Listening to connections...");

  const fileServer = options.serveFiles ? createFileServer() : null;
  await arenaServer({
    async fetch(req) {
      if (fileServer) {
        let file = await fileServer.fetch(req);
        if (file.status !== 404) {
          return file;
        }
      }
      return await handler.fetch(req);
    },
    websocket: handler.websocket,
  });
};

export { serve };
