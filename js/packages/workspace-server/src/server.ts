import type { Handler } from "@arena/core/server";
import { ArenaRequest, RESOLVE } from "./request";
import { createFileServer } from "./fileserver";

class RequestListener {
  [Symbol.asyncIterator]() {
    return this;
  }

  async next() {
    try {
      const req = await Arena.core.opAsync("op_receive_request");
      return { value: req, done: false };
    } catch (error) {
      console.error(error);
      // TODO(sagar): handle error
      return { value: undefined, done: true };
    }
  }
}

const serve = async (
  handler: Handler,
  options: { serveFiles?: boolean } = {}
) => {
  // Note(sagar): since this is running in server, set SSR = true
  Arena.env.SSR = true;

  // TODO(sagar): we need to store logs from Arena and logs from queries
  // separately
  console.log("[Arena.Workspace.serve]: Listening to connections...");

  const fileServer = options.serveFiles ? createFileServer() : null;
  const listener = new RequestListener();
  for await (const req of listener) {
    if (!req) {
      break;
    }
    let arenaRequest = new ArenaRequest(req.internal, req.rid);
    arenaRequest[RESOLVE](async () => {
      let event = {
        request: arenaRequest,
        env: Arena.env,
      };

      if (fileServer) {
        let file = await fileServer.execute(event);
        if (file.status !== 404) {
          return file;
        }
      }
      return await handler.execute(event);
    });
  }
};

export { serve };
