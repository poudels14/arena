import { trough } from "trough";
// @ts-ignore
import qs from "query-string";
// @ts-ignore
import cookie from "cookie";
import { PageEvent } from "./event";

type Middleware<Arg> = (arg: Arg) => Promise<Response | void>;

interface Websocket extends AsyncIterator<any> {
  send(data: any): Promise<number>;
  close(data?: any): Promise<void>;
  next(): Promise<any>;
}

type Handler = {
  fetch: (req: Request) => Promise<Response>;
  websocket?: (websocket: Websocket, data: any) => void;
};

const createPageEvent = (request: Request): PageEvent => {
  let url = new URL(request.url);
  const cookies = cookie.parse(request.headers.get("Cookie") || "");
  return {
    request,
    env: process.env,
    cookies,
    ctx: {
      path: url.pathname,
      search: url.search,
      query: qs.parse(url.search) as Record<string, string>,
    },
    tags: [],
  };
};

/**
 * Chain bunch of middlewares
 *
 * This allows passing `null` as a middleware so that a middleware
 * can be chained conditionally.
 */
function chainMiddlewares<T>(...middlewares: (Middleware<T> | null)[]) {
  const pipeline = middlewares
    .filter((m) => Boolean(m))
    .reduce((t, m) => {
      return t.use(m!).use((r) => {
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

  return (e: T) => {
    return new Promise<Response>((resolve, reject) => {
      pipeline.run(e, (err: any, data: any) => {
        // Note(sagar): if either data or error is of type Response,
        // return it early. Else, wrap it with Response and return it
        if (err instanceof Response) {
          return resolve(err);
        } else if (data instanceof Response) {
          return resolve(data);
        }

        if (err) {
          console.error(err);
          resolve(
            new Response(null, {
              status: 503,
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
  };
}

const createHandler = (...middlewares: Middleware<PageEvent>[]): Handler => {
  const pipeline = chainMiddlewares(...middlewares);

  return {
    fetch(event: Request) {
      const pageEvent = createPageEvent(event);
      return pipeline(pageEvent);
    },
  };
};

export type { Handler };
export { chainMiddlewares, createHandler };
