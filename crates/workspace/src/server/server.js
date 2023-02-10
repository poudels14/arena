"use strict";

((global) => {
  if (!global.Arena) {
    global.Arena = {};
  };

  const { ops } = Deno.core;

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

  const RESOLVE = Symbol("_request_resolver_");

  class ArenaRequest extends Request {
    constructor(internal, response_rid) {
      super(internal.url);
      this.rid = response_rid;
    }

    async [RESOLVE](asyncFn) {
      let res = await asyncFn()
      this.responseWith(res);
    }

    async responseWith(response) {
      // TODO(sagar): consider not using Deno's Request/Response type, too
      // much going on there
      let innerResponse = Arena.toInnerResponse(response);

      let content = innerResponse.body.streamOrStatic?.body || innerResponse.body.source;
      // TODO(sagar): throw error if stream is used
      // if (content == undefined) {
      //   throw new Error("Stream response not supported!");
      // }
      await ops.op_send_response(this.rid, innerResponse.status, innerResponse.headerList, content);
    }
  }

  const Workspace = {
    async handleRequest(handler) {
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
            request: arenaRequest
          });
          if (res.then) {
            res = await res;
          }
          return res;
        })  
      }
    }
  };

  globalThis.Arena.Workspace = Workspace;
  global.ArenaRequest = ArenaRequest;
  
})(globalThis);
