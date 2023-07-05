// @ts-expect-error
import { CookieSerializeOptions } from "cookie";

// Note(sp): this is to prevent env replacement during build
const isDev = () => {
  const env = process.env;
  return env["NODE_ENV"] == "development";
};

type ProcedureCallbackArgs<Context> = {
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
  next: (args: Partial<ProcedureCallbackArgs<Context>>) => void;
};

type ProcedureCallback<Context> = (
  args: ProcedureCallbackArgs<Context>
) => Promise<any> | any;

type Handler<Context> = {
  (args: Omit<ProcedureCallbackArgs<Context>, "next">): Promise<Response>;
  method: "GET" | "POST" | "PATCH";
};

const createHandler =
  <Context>(fns: ProcedureCallback<Context>[]) =>
  async (reqArgs: ProcedureCallbackArgs<Context>) => {
    const generateNextFn = (currIdx: number) => {
      return (
        nextArgs: Partial<Omit<ProcedureCallbackArgs<Context>, "next">>
      ) =>
        fns[currIdx + 1]({
          ...reqArgs,
          ctx: {
            ...reqArgs.ctx,
            ...(nextArgs.ctx || {}),
          },
          next: generateNextFn(currIdx + 1),
        });
    };

    let response;
    try {
      response = fns[0]({ ...reqArgs, next: generateNextFn(0) });
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

const createProcedure = <Context>(fns: ProcedureCallback<Context>[]) => {
  return {
    use(cb: ProcedureCallback<Context>) {
      return createProcedure([...fns, cb]);
    },
    query(cb: ProcedureCallback<Context>) {
      return Object.assign(createHandler([...fns, cb]), {
        method: "GET" as const,
      }) as Handler<Context>;
    },
    mutate(cb: ProcedureCallback<Context>) {
      return Object.assign(createHandler([...fns, cb]), {
        method: "POST" as const,
      }) as Handler<Context>;
    },
    patch(cb: ProcedureCallback<Context>) {
      return Object.assign(createHandler([...fns, cb]), {
        method: "PATCH" as const,
      }) as Handler<Context>;
    },
  };
};

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
  return createProcedure<Context>([]);
};

export { procedure };
export type { Handler };
