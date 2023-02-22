import { trough } from "trough";
import { PageEvent } from "./event";

type Middleware = (event: PageEvent) => Promise<Response | void>;

type Handler = {
  execute: (event: PageEvent) => Promise<Response>;
};

const createHandler = (...middlewares: Middleware[]): Handler => {
  const pipeline = middlewares.reduce((t, m) => {
    return t.use(m).use((r) => {
      // Note(sagar): if the middleware returns a response, stop executing
      // rest of the middlewares and send response
      if (r instanceof Response) {
        throw r;
      }
    });
  }, trough());

  /**
   * If the middlewares don't return a response, return 404 as default response
   */
  pipeline.use((_) => {
    return new Response(null, {
      status: 404,
    });
  });

  return {
    /**
     * This returns the Response object
     */
    execute(event: PageEvent) {
      return new Promise((resolve, reject) => {
        pipeline.run(event, (err: any, data: any) => {
          // Note(sagar): if either data or error is of type Response,
          // return it early. Else, wrap it with Response and return it
          if (err instanceof Response) {
            return resolve(err);
          } else if (data instanceof Response) {
            return resolve(data);
          }

          if (err) {
            resolve(
              new Response(null, {
                status: 500,
                // TODO(sagar): use some library to get proper error messages
                statusText: "Internal Server Error",
              })
            );
          } else {
            resolve(
              new Response(data, {
                status: 200,
              })
            );
          }
        });
      });
    },
  };
};

export type { Handler };
export { createHandler };
