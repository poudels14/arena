import { mergeMap, from } from "rxjs";
import { Server } from "./tcp";
import { Websocket } from "./websocket";

type ServeConfig = {
  fetch: (req: Request) => Promise<Response>;
  websocket?: (websocket: Websocket, data: any) => Promise<void>;
};

const serve = async (config: ServeConfig) => {
  const server = await Server.init();
  const streams = from(server);
  streams.pipe(mergeMap((stream) => from(stream!))).subscribe(async (req) => {
    try {
      let res = await config.fetch(req!);
      let response =
        res instanceof Response
          ? res
          : new Response(res, {
              status: 200,
            });

      req!.send(response).then((result) => {
        if (result && config.websocket) {
          config.websocket(result[0], result[1]);
        }
      });
    } catch (error) {
      console.error(error);
      req!.send(
        new Response("Internal Server Error", {
          status: 500,
        })
      );
    }
  });
};

export { serve };
export * from "@arena/core/router";
