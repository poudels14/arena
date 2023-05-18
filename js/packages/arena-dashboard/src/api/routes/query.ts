import { z } from "zod";
import { pick } from "lodash-es";
import { DqsServer, DqsCluster } from "@arena/runtime/dqs";
import { procedure, router as trpcRouter } from "../trpc";

const dqsCluster = new Map<string, DqsServer>();
const queryRouter = trpcRouter({
  fetch: procedure
    .input(
      z.object({
        workspaceId: z.string(),
        appId: z.string(),
        widgetId: z.string(),
        field: z.string(),
        params: z.record(z.any()),
      })
    )
    .query(async ({ ctx, input }): Promise<any> => {
      const res = await pipeRequestToDqs("QUERY", input);
      // TODO(sagar): find a way to send response without converting to JSON
      return JSON.parse(String.fromCharCode.apply(null, res[2]));
    }),
  mutate: procedure
    .input(
      z.object({
        workspaceId: z.string(),
        appId: z.string(),
        widgetId: z.string(),
        field: z.string(),
        params: z.record(z.any()),
      })
    )
    .query(async ({ ctx, input }): Promise<any> => {
      const res = await pipeRequestToDqs("MUTATION", input);
      // TODO(sagar): find a way to send response without converting to JSON
      return JSON.parse(String.fromCharCode.apply(null, res[2]));
    }),
});

type Input = (typeof queryRouter.fetch._def)["_input_in"];
const pipeRequestToDqs = async (
  trigger: "QUERY" | "MUTATION",
  input: Input
) => {
  const { workspaceId } = input;
  let server = dqsCluster.get(workspaceId);
  if (!server || !server.isAlive()) {
    console.log("starting server for workspace =", workspaceId);
    server = await DqsCluster.startStreamServer(workspaceId);
    dqsCluster.set(workspaceId, server);
  }

  return await server.pipeRequest({
    url: "http://0.0.0.0/execWidgetQuery",
    method: "POST",
    headers: [["content-type", "application/json"]],
    body: {
      trigger,
      ...pick(input, "workspaceId", "appId", "widgetId", "field", "params"),
    },
  });
};

export { queryRouter };
