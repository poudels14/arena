import { PageEvent } from "@arena/core/server/event";
import { fetchRequestHandler } from "@trpc/server/adapters/fetch";
import { mergedRouter } from "@arena/runtime/server";
import { Context } from "./context";
import { procedure, router as trpcRouter } from "./trpc";
import { appsRouter } from "./routes/apps";
import { widgetsRouter } from "./routes/widgets";
import { resourcesRouter } from "./routes/resources";
import { queryRouter } from "./routes/query";
import { accountRouter } from "./routes/account";

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
  const findMyRouter = mergedRouter({
    prefix: "/api",
    routers: [queryRouter, accountRouter],
  });

  return async ({ event, context }: { event: PageEvent; context: Context }) => {
    if (options.prefix && !event.ctx.path.startsWith(options.prefix)) {
      return;
    }

    let res;
    if ((res = await findMyRouter.route(event.request, context))) {
      return res;
    }

    return await fetchRequestHandler({
      endpoint: options.prefix || "",
      req: event.request,
      router: r,
      createContext() {
        return context;
      },
      onError(e) {
        console.error(e.error);
      },
    });
  };
};

export type AppRouter = typeof r;
export { router };
