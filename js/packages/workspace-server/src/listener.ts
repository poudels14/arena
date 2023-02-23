import type { Handler } from "@arena/core/server";
import qs from "query-string";
import { ArenaRequest, RESOLVE } from "./request";
import fileServer from "./fileserver";

const { ops } = Arena.core;

class RequestListener {
  [Symbol.asyncIterator]() {
    return this;
  }

  async next() {
    try {
      const req = await ops.op_receive_request();
      return { value: req, done: false };
    } catch (error) {
      console.error(error);
      // TODO(sagar): handle error
      return { value: undefined, done: true };
    }
  }
}

const serve = async (handler: Handler) => {
  // TODO(sagar): we need to store logs from Arena and logs from queries
  // separately
  console.log("[Arena.Workspace.handleRequest]: Listening to connections...");

  const listener = new RequestListener();
  for await (const req of listener) {
    if (!req) {
      break;
    }
    let arenaRequest = new ArenaRequest(req.internal, req.rid);
    arenaRequest[RESOLVE](async () => {
      let url = new URL(req.internal.url);

      let event = {
        request: arenaRequest,
        env: Arena.env,
        ctx: {
          path: url.pathname,
          query: qs.parse(url.search),
        },
      };

      let file = await fileServer.execute(event);
      if (file.status !== 404) {
        return file;
      }
      return await handler.execute(event);
    });
  }
};

export { serve };
