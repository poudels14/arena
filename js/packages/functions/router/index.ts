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
        // else MUTATION
        // trigger type MUTATION is expected to mutate data in remote data source
        trigger: z.enum(["QUERY", "MUTATION"]),
        workspaceId: z.string(),
        appId: z.string(),
        widgetId: z.string(),
        field: z.string(),
        // the last updated time of the widget so that to reload
        // data query if needed
        updatedAt: z.string(),
        params: z.record(z.any()).optional(),
      })
    )
    .mutation(
      async ({
        input: { workspaceId, appId, widgetId, field, updatedAt, params },
      }) => {
        try {
          const env = await import(
            `~/apps/${appId}/widgets/${widgetId}/${field}/env`
          );
          return await import(
            `~/apps/${appId}/widgets/${widgetId}/${field}?updatedAt=${updatedAt}`
          ).then(async (m) => {
            const result = await Promise.all([
              m.default({
                params: params || {},
                env,
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
