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

type RouterConfig<Context> = {
  host?: string;
  prefix?: string;
  defaultHandler?: ProcedureCallback<Context>;
};

const createRouter = <Context>(
  config: RouterConfig<Context> & {
    routes: Record<string, Handler<Context>>;
  }
) => {
  const r = findMyWay();
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

      let routeHandler = (route && route.handler) || config.defaultHandler;
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

        // @ts-expect-error
        request.cookies = parseCookie(request.headers.get("Cookie") || "");
        // @ts-expect-error
        const res: Response = await routeHandler({
          req: request,
          env: meta.env || {},
          ctx: meta.context || {},
          params: route?.params || {},
          searchParams: route?.searchParams || {},
          errors,
          setHeader,
          redirect,
          setCookie,
          clearCookie,
          form: {
            multipart: parseFormData,
          },
        });

        _resInternal.headers.forEach((h) => {
          res.headers.set(h[0], h[1]);
        });

        _resInternal.cookies.forEach(([name, value, options]) => {
          res.headers.set("Set-Cookie", serializeCookie(name, value, options));
        });

        return res;
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
    routers: ReturnType<typeof createRouter>[];
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
