import { PageEvent } from "@arena/core/server/event";
import { fetchRequestHandler } from "@trpc/server/adapters/fetch";
import { mergedRouter } from "@arena/runtime/server";
import { Context } from "./context";
import { procedure, router as trpcRouter } from "./trpc";
import { appsRouter } from "./routes/apps";
import { widgetsRouter } from "./routes/widgets";
import { resourcesRouter } from "./routes/resources";
import { accountRouter } from "../../../../workspace-cluster/api/account";
import { pluginsRouter } from "./routes/plugins";
import { workflowsRouter } from "./routes/workflows";

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
    ignoreTrailingSlash: true,
    async middleware({ ctx, next }) {
      try {
        return await next({ ctx });
      } catch (e) {
        console.error(e);
        return e;
      }
    },
    routers: [accountRouter, pluginsRouter, workflowsRouter],
  });

  return async ({ event, context }: { event: PageEvent; context: Context }) => {
    if (options.prefix && !event.ctx.path.startsWith(options.prefix)) {
      return;
    }

    let res;
    if ((res = await findMyRouter.route(event.request, { context }))) {
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
