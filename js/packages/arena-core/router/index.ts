import findMyWay, { HTTPMethod } from "find-my-way";

type RouterConfig = {
  host?: string;
};

const router = (config: RouterConfig) => {
  const routes = findMyWay();
  return Object.assign(routes, {
    async route(request: Request) {
      const route = routes.find(request.method as HTTPMethod, request.url, {
        host: config.host,
      });

      if (route && route.handler) {
        const res = response();
        await route.handler(
          // @ts-expect-error
          request,
          res,
          route.params,
          route.store,
          route.searchParams
        );
        return res.getResponse();
      }
    },
  });
};

const response = (): any => {
  const internal: any = {
    status: 200,
    body: null,
    headers: [["content-type", "application/json"]],
    _response: null,
  };

  return {
    statusCode(status: number) {
      internal.status = status;
    },
    setHeader(name: string, value: string) {
      name = name.toLowerCase();
      internal.headers.filter((h: any) => h[0] != name).push([name, value]);
    },
    end(data: any) {
      internal.body = data;
    },
    sendResponse(res: unknown) {
      if (res instanceof Response) {
        internal._response = res;
      }
      internal.body = res;
    },
    getResponse() {
      return (
        internal._response ??
        new Response(JSON.stringify(internal.body), {
          status: internal.status,
          headers: new Headers(internal.headers),
        })
      );
    },
  };
};

export { router };
