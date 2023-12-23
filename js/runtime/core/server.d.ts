interface Websocket extends AsyncIterator<any> {
  /**
   * returns 0 if sending message failed
   */
  send(data: any): Promise<number>;
  close(data?: any): Promise<void>;
  next(): Promise<any>;
}

type ServeConfig = {
  fetch: (req: Request) => Promise<Response>;
  websocket?: (websocket: Websocket, data: any) => void;
};

type Handler<Context> = {
  (args: ProcedureCallbackArgs<Context>): Promise<Response>;
  method: "GET" | "POST" | "PATCH";
};

type RouterConfig<Context> = {
  host?: string;
  prefix?: string;
  ignoreTrailingSlash?: boolean;
  ignoreDuplicateSlashes?: boolean;

  defaultHandler?: ProcedureCallback<Context>;

  /**
   * A middleware similar to procedure middleware but applies to
   * all routes under this router. This can be used to setup higher level
   * middleware if the router doesn't have access to route level procedures
   * or has sub-routers with different procedures
   */
  middleware?: ProcedureCallback<Context>;
};

type ProcedureCallbackArgs<Context> = {
  req: Request;
  env: any;
  ctx: Context;
  params: Record<string, string>;
  searchParams: Record<string, string>;
  cookies: Record<string, string>;
  errors: {
    notFound(message?: any): void;
    badRequest(message?: any): void;
    forbidden(message?: any): void;
    internalServerError(message?: any): void;
  };
  setHeader: (name: string, value: string) => void;
  setCookie(
    name: string,
    value: string,
    options?: {
      domain?: string | undefined;
      /**
       * The default function is the global `encodeURIComponent`
       */
      encode?(value: string): string;
      /**
       * By default, no expiration is set, and most clients will delete it after session expires
       */
      expires?: Date | undefined;
      /**
       * By default, the `HttpOnly` attribute is not set.
       */
      httpOnly?: boolean | undefined;
      maxAge?: number | undefined;
      /**
       * By default, the path is considered the "default path".
       */
      path?: string | undefined;
      priority?: "low" | "medium" | "high" | undefined;
      sameSite?: true | false | "lax" | "strict" | "none" | undefined;
      /**
       * By default, the `Secure` attribute is not set.
       */
      secure?: boolean | undefined;
    }
  ): void;
  clearCookie: (name: string) => void;
  form: {
    /**
     * Parse request for multipart form data
     */
    multipart: (req: Request) => Promise<
      {
        filename: string;
        type: string;
        name: string;
        data: Buffer;
      }[]
    >;
  };
  redirect: (path: string) => void;
  next: (args: Partial<ProcedureCallbackArgs<Context>>) => void;
};

type ProcedureCallback<Context> = (
  args: ProcedureCallbackArgs<Context>
) => Promise<any> | any;

type Procedure<Context> = {
  use(cb: ProcedureCallback<Context>): Procedure<Context>;
  query(cb: ProcedureCallback<Context>): Handler<Context>;
  mutate(cb: ProcedureCallback<Context>): Handler<Context>;
  patch(cb: ProcedureCallback<Context>): Handler<Context>;
  delete(cb: ProcedureCallback<Context>): Handler<Context>;
};

export const procedure: <Context>() => Procedure<Context>;

export const createRouter: <Context>(
  config: RouterConfig<Context> & {
    routes: Record<string, Handler<Context>>;
  }
) => {
  route(
    request: Request,
    meta?: { env?: Record<string, string | undefined>; context?: Context }
  ): Promise<Response>;
};

export const mergedRouter: <Context>(
  config: RouterConfig<Context> & {
    prefix?: string;
    routers: ReturnType<typeof createRouter<Context>>[];
  }
) => ReturnType<typeof createRouter<Context>>;

export const serve: (config: ServeConfig) => Promise<void>;
