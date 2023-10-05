import findMyWay, { HTTPMethod } from "find-my-way";
import nodePath from "path";
import {
  serialize as serializeCookie,
  parse as parseCookie,
  CookieSerializeOptions,
  // @ts-expect-error
} from "cookie";
import { Handler, ProcedureCallback } from "./procedure";
import { errors } from "./errors";
import { parseFormData } from "./formdata";
import { generateResponse } from "./response";

type RouterConfig<Context> = {
  host?: string;
  prefix?: string;
  ignoreTrailingSlash?: boolean;
  ignoreDuplicateSlashes?: boolean;

  defaultHandler?: Handler<Context>;

  /**
   * A middleware similar to procedure middleware but applies to
   * all routes under this router. This can be used to setup higher level
   * middleware if the router doesn't have access to route level procedures
   * or has sub-routers with different procedures
   */
  middleware?: ProcedureCallback<Context>;
};

const createRouter = <Context>(
  config: RouterConfig<Context> & {
    routes: Record<string, Handler<Context>>;
  }
) => {
  const r = findMyWay({
    ignoreTrailingSlash: config.ignoreTrailingSlash,
    ignoreDuplicateSlashes: config.ignoreDuplicateSlashes,
  });

  const routes = Object.entries(config.routes).map(
    ([path, handler]) =>
      [nodePath.join(config.prefix || "/", path), handler] as const
  );
  routes.forEach(([path, handler]) => {
    r.on(
      handler.method,
      path,
      // @ts-expect-error
      handler
    );
  });

  return {
    async route(
      request: Request,
      meta: {
        context?: Context;
        env?: Record<string, string>;
      } = {}
    ) {
      const route = r.find(request.method as HTTPMethod, request.url, {
        host: config.host,
      });

      let routeHandler =
        ((route && route.handler) as Omit<Handler<Context>, "method">) ||
        config.defaultHandler;
      if (routeHandler) {
        const _resInternal = {
          headers: [] as unknown as [string, string][],
          cookies: [] as unknown as [
            string,
            string,
            CookieSerializeOptions | undefined
          ][],
        };

        const setHeader = (name: string, value: string) => {
          name = name.toLowerCase();
          _resInternal.headers = _resInternal.headers.filter(
            (h: any) => h[0] != name
          );
          _resInternal.headers.push([name, value]);
        };

        const redirect = (path: string) => {
          setHeader("Location", path);
          return new Response("302 Found", {
            status: 302,
          });
        };

        const setCookie = (
          name: string,
          value: string,
          options?: CookieSerializeOptions
        ) => {
          _resInternal.cookies = _resInternal.cookies.filter(
            (h: any) => h[0] != name
          );
          _resInternal.cookies.push([name, value, options]);
        };

        const clearCookie = (name: string) => {
          _resInternal.cookies.push([
            name,
            "",
            {
              expires: new Date(0),
            },
          ]);
        };

        let response;
        try {
          // @ts-expect-error
          response = routeHandler(
            {
              req: request,
              env: meta.env || {},
              ctx: meta.context || {},
              params: route?.params || {},
              searchParams: route?.searchParams || {},
              errors,
              setHeader,
              redirect,
              cookies: parseCookie(request.headers.get("Cookie") || ""),
              setCookie,
              clearCookie,
              form: {
                multipart: parseFormData,
              },
            },
            {
              middlewares: config.middleware ? [config.middleware] : null,
            }
          );
          if (response?.then) {
            response = await response;
          }
        } catch (e) {
          response = e;
        }

        /**
         * Only return a valid response, else return undefined so that
         * server can return proper error message. Alternatively, the
         * router can be chained with some other handler if undefined
         * is returned
         */
        if (response) {
          const res: Response = generateResponse(response);

          _resInternal.headers.forEach((h) => {
            res.headers.set(h[0], h[1]);
          });

          _resInternal.cookies.forEach(([name, value, options]) => {
            res.headers.set(
              "Set-Cookie",
              serializeCookie(name, value, options)
            );
          });

          return res;
        }
        return undefined;
      }
    },
    listRoutes() {
      return Object.fromEntries([...routes]);
    },
    reset() {
      r.reset();
    },
  };
};

const mergedRouter = <Context>(
  option: RouterConfig<Context> & {
    routers: ReturnType<typeof createRouter<Context>>[];
  }
) => {
  const routes = Object.fromEntries(
    option.routers.flatMap((r) => Object.entries(r.listRoutes()))
  );
  const router = createRouter({
    ...option,
    routes,
  });
  return router;
};

export { createRouter, mergedRouter };
export { procedure } from "./procedure";
export { parseFormData } from "./formdata";
