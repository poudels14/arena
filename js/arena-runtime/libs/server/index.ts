import { mergeMap, from } from "rxjs";
import { Server } from "./tcp";

type ServeConfig = {
  fetch: (req: Request) => Promise<Response>;
};

const serve = async (config: ServeConfig) => {
  const server = await Server.init();
  const streams = from(server);
  streams.pipe(mergeMap((stream) => from(stream!))).subscribe(async (req) => {
    try {
      let res = await config.fetch(req!);
      if (res instanceof Response) {
        req!.send(res);
      } else {
        req!.send(
          new Response(String(res), {
            status: 200,
          })
        );
      }
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
