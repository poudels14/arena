import { z } from "zod";
import { createRouter, procedure } from "@arena/core/router";

const execWidgetQueryBodySchema = z.object({
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
  props: z.record(z.any()).optional(),
});

const p = procedure();
const router = createRouter({
  routes: {
    "/healthy": p.query(() => {
      return "Ok";
    }),
    "/:appId/widgets/:widgetId/api/:field": p.mutate(
      async ({ req, params }) => {
        const { workspaceId, appId, widgetId, field, updatedAt, props } =
          execWidgetQueryBodySchema.parse(await req.json());
        try {
          const env = await import(
            `~/apps/${appId}/widgets/${widgetId}/${field}/env`
          );
          return await import(
            `~/apps/${appId}/widgets/${widgetId}/${field}?updatedAt=${updatedAt}`
          ).then(async (m) => {
            const result = await Promise.all([
              m.default({
                props: props || {},
                env,
              }),
            ]);
            return result[0];
          });
        } catch (e) {
          return e;
        }
      }
    ),
  },
});

export { router };
