import { PageEvent } from "@arena/core/server/event";
import { fetchRequestHandler } from "@trpc/server/adapters/fetch";
import { createContext } from "./context";
import { procedure, router as trpcRouter } from "./trpc";
import { appsRouter } from "./routes/apps";
import { widgetsRouter } from "./routes/widgets";
import { resourcesRouter } from "./routes/resources";
import { queryRouter } from "./routes/query";

const r = trpcRouter({
  apps: appsRouter,
  widgets: widgetsRouter,
  resources: resourcesRouter,
  _healthy: procedure.query(() => {
    return "OK!";
  }),
});

type RouterOptions = {
  /**
   * Router path prefix
   */
  prefix?: string;
};

const router = (options: RouterOptions) => {
  return async (event: PageEvent) => {
    if (options.prefix && !event.ctx.path.startsWith(options.prefix)) {
      return;
    }

    let res;
    if ((res = await queryRouter.route(event.request))) {
      return res;
    }

    return await fetchRequestHandler({
      endpoint: options.prefix || "",
      req: event.request,
      router: r,
      createContext,
      onError(e) {
        console.error(e.error);
      },
    });
  };
};

export type AppRouter = typeof r;
export { router };
