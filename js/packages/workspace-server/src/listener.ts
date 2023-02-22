import { ArenaRequest, RESOLVE } from "./request";

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

const serve = async (handler: any) => {
  // TODO(sagar): get rid of this. using this to keep V8 event loop alive
  setTimeout(() => {}, 100_000);

  // TODO(sagar): we need to store logs from Arena and logs from queries
  // separately
  console.log("[Arena.Workspace.handleRequest]: Listening to connections...");

  const listener = new RequestListener();

  for await (const req of listener) {
    let arenaRequest = new ArenaRequest(req.internal, req.rid);
    arenaRequest[RESOLVE](async () => {
      let res = handler.call(handler, {
        request: arenaRequest,
      });
      if (res.then) {
        res = await res;
      }
      return res;
    });
  }
};

export { serve };
