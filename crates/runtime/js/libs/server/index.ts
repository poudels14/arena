import { mergeMap, from } from "rxjs";
import { Server } from "./tcp";
import { Websocket } from "./websocket";

type ServeConfig = {
  // passing in env and context arguments provides compatibility
  // with cloudflare workers runtime
  fetch: (
    req: Request,
    env: any,
    context: {
      waitUntil(promise: Promise<any>): void;
      passThroughOnException: () => void;
    }
  ) => Promise<Response>;
  websocket?: (websocket: Websocket, data: any) => Promise<void>;
};

const serve = async (config: ServeConfig) => {
  // Catch global errors to avoid crasing the server
  globalThis.addEventListener("error", (e) => {
    console.error(e);
    e.preventDefault();
    e.stopPropagation();
  });

  const server = await Server.init();
  const streams = from(server);
  streams.pipe(mergeMap((stream) => from(stream!))).subscribe(async (req) => {
    try {
      let res = await config.fetch(req!, process.env, {
        waitUntil() {},
        passThroughOnException() {},
      });
      if (!res) {
        return new Response("Not found", {
          status: 404,
        });
      }
      let response =
        res instanceof Response
          ? res
          : new Response(res, {
              status: 200,
            });

      await req!
        .send(response)
        .then((result) => {
          if (result && config.websocket) {
            config.websocket(result[0], result[1]);
          }
        })
        .catch((e) =>
          console.error(`Error sending response [path = ${req?.url}]`)
        );
    } catch (error) {
      console.error(error);
      req!
        .send(
          new Response("Internal Server Error", {
            status: 500,
          })
        )
        .catch(() => {
          // ignore error
        });
    }
  });
};

export { serve };
