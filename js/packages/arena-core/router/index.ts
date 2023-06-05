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
    headers: [],
    _response: null,
  };

  return {
    statusCode(status: number) {
      internal.status = status;
    },
    setHeader(name: string, value: string) {
      internal.headers.push([name, value]);
    },
    end(data: any) {
      if (typeof data == "object") {
        internal.headers.push(["content-type", "application/json"]);
      }
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
        })
      );
    },
  };
};

export { router };
