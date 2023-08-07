// @ts-expect-error
import { CookieSerializeOptions } from "cookie";

// Note(sp): this is to prevent env replacement during build
const isDev = () => {
  const env = process.env;
  return env["NODE_ENV"] == "development";
};

type RequestEvent<Context> = {
  req: Request;
  env: Record<string, string>;
  ctx: Context;
  setCookie: (
    name: string,
    value: string,
    options?: CookieSerializeOptions
  ) => void;
  /**
   * Path params
   */
  params: Record<string, string>;

  /**
   * Search/Query params
   */
  searchParams: Record<string, string>;
  cookies: Record<string, string>;
  errors: {
    notFound(): void;
    badRequest(): void;
    forbidden(): void;
    internalServerError(): void;
  };
  setHeader: (name: string, value: string) => void;
  clearCookie: (name: string) => void;
  redirect: (path: string) => void;
  next: (event: Partial<RequestEvent<Context>>) => void;
};

type ProcedureCallback<Context> = (
  event: RequestEvent<Context>
) => Promise<any> | any;

type HandleOptions<Context> = {
  middlewares: ProcedureCallback<Context>[];
};

type Handler<Context> = {
  (
    event: Omit<RequestEvent<Context>, "next">,
    options?: HandleOptions<Context>
  ): Promise<Response>;
  method: "GET" | "POST" | "PATCH";
};

const createHandler = <Context>(
  fns: ProcedureCallback<Context>[]
): Omit<Handler<Context>, "method"> => {
  return async (
    event: RequestEvent<Context>,
    options: HandleOptions<Context>
  ) => {
    fns = options?.middlewares ? [...options.middlewares, ...fns] : fns;
    const generateNextFn = (currIdx: number) => {
      return (nextArgs: Partial<Omit<RequestEvent<Context>, "next">>) =>
        fns[currIdx + 1]({
          ...event,
          ctx: {
            ...event.ctx,
            ...(nextArgs.ctx || {}),
          },
          next: generateNextFn(currIdx + 1),
        });
    };

    let response;
    try {
      response = fns[0]({ ...event, next: generateNextFn(0) });
    } catch (e) {
      response = e;
    }

    if (!response) {
      return generateResponse(new Error("Middleware should return"));
    } else if (response instanceof Promise) {
      response = await response.catch((e) => e);
    }
    return generateResponse(response);
  };
};

class Procedure<Context> {
  fns: ProcedureCallback<Context>[];
  constructor(fns: ProcedureCallback<Context>[]) {
    this.fns = fns;
  }

  use(cb: ProcedureCallback<Context>) {
    return new Procedure([...this.fns, cb]);
  }

  query(cb: ProcedureCallback<Context>) {
    return this.handle("GET", cb);
  }

  mutate(cb: ProcedureCallback<Context>) {
    return this.handle("POST", cb);
  }

  patch(cb: ProcedureCallback<Context>) {
    return this.handle("PATCH", cb);
  }

  delete(cb: ProcedureCallback<Context>) {
    return this.handle("DELETE", cb);
  }

  private handle(
    method: "GET" | "POST" | "PATCH" | "DELETE",
    cb: ProcedureCallback<Context>
  ) {
    return Object.assign(createHandler([...this.fns, cb]), {
      method,
    }) as Handler<Context>;
  }
}

const generateResponse = (response: any) => {
  if (response instanceof Response) {
    return response;
  } else if (response instanceof Error) {
    if (isDev()) {
      return jsonResponse(500, {
        error: {
          cause: response.cause,
          stack: response.stack,
        },
      });
    }
    return new Response("500 Internal Server Error", {
      status: 500,
    });
  } else {
    if (
      typeof response != "string" &&
      !(response instanceof Uint8Array) &&
      !(response instanceof Uint16Array)
    ) {
      return jsonResponse(200, response);
    }
    return new Response(response, {
      status: 200,
    });
  }
};

const jsonResponse = (status: number, response: any) => {
  return new Response(JSON.stringify(response), {
    status,
    headers: new Headers([["content-type", "application/json"]]),
  });
};

const procedure = <Context>() => {
  return new Procedure<Context>([]);
};

export { procedure };
export type { Handler, ProcedureCallback };
