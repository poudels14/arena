import { fetchRequestHandler } from "@trpc/server/adapters/fetch";
import { z } from "zod";
import { createContext } from "./context";
import { procedure, router as trpcRouter } from "./trpc";

const r = trpcRouter({
  healthy: procedure.query(() => {
    return "OK";
  }),
  // execSavedQuery
  // execQuery
  execWidgetQuery: procedure
    .input(
      z.object({
        // the trigger is QUERY if the data query exec was triggered by GET
        // else ACTION
        // trigger type ACTION is expected to mutate data in remote data source
        trigger: z.enum(["QUERY", "ACTION"]),
        workspaceId: z.string(),
        appId: z.string(),
        widgetId: z.string(),
        field: z.string(),
        params: z.record(z.any()).optional(),
      })
    )
    .mutation(
      async ({ input: { workspaceId, appId, widgetId, field, params } }) => {
        try {
          return await import(
            `~/apps/${appId}/widgets/${widgetId}/${field}`
          ).then(async (m) => {
            const result = await Promise.all([
              m.default({
                params: params || {},
              }),
            ]);
            return result[0];
          });
        } catch (e) {
          console.error(e);
          throw e;
        }
      }
    ),
});

type RouterConfig = {
  workspaceId: string;
};

const router = (config: RouterConfig) => {
  return {
    route: async (request: Request) => {
      return await fetchRequestHandler({
        endpoint: "",
        req: request,
        router: r,
        createContext,
      });
    },
  };
};

export type DqsRouter = typeof r;
export { router };
