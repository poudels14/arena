import { z } from "zod";
import { router as createRouter } from "@arena/core/router";

const r = createRouter({});

r.on("GET", "/healthy", (req, res, params) => {
  res.send("OK");
});

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
  params: z.record(z.any()).optional(),
});

r.on("POST", "/execWidgetQuery", async (req, res) => {
  const { workspaceId, appId, widgetId, field, updatedAt, params } =
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
          params: params || {},
          env,
        }),
      ]);
      return res.end(result[0]);
    });
  } catch (e) {
    console.error(e);
    throw e;
  }
});

const router = () => {
  return {
    async route(request: Request) {
      const res = await r.route(request);
      if (res) {
        return res;
      }

      return new Response("Not found", {
        status: 404,
      });
    },
  };
};

export { router };
