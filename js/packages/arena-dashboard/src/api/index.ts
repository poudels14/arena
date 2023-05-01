import { PageEvent } from "@arena/core/server/event";
import { fetchRequestHandler } from "@trpc/server/adapters/fetch";
import { createContext } from "./context";
import { router as trpcRouter } from "./trpc";
import { appsRouter } from "./routes/apps";
import { widgetsRouter } from "./routes/widgets";

const r = trpcRouter({
  apps: appsRouter,
  widgets: widgetsRouter,
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

    return await fetchRequestHandler({
      endpoint: options.prefix || "/",
      req: event.request,
      router: r,
      createContext,
    });
  };
};

export type AppRouter = typeof r;
export { router };
