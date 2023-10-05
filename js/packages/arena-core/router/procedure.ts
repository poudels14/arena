// @ts-expect-error
import { CookieSerializeOptions } from "cookie";

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
    const finalFns = options?.middlewares
      ? [...options.middlewares, ...fns]
      : [...fns];
    const generateNextFn = (currIdx: number) => {
      return (nextArgs: Partial<Omit<RequestEvent<Context>, "next">>) =>
        finalFns[currIdx + 1]({
          ...event,
          ctx: {
            ...event.ctx,
            ...(nextArgs.ctx || {}),
          },
          next: generateNextFn(currIdx + 1),
        });
    };
    return finalFns[0]({ ...event, next: generateNextFn(0) });
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

const procedure = <Context>() => {
  return new Procedure<Context>([]);
};

export { procedure };
export type { Handler, ProcedureCallback };
