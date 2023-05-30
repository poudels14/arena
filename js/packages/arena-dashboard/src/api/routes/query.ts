import { z } from "zod";
import { pick } from "lodash-es";
import { DqsServer, DqsCluster } from "@arena/runtime/dqs";
import { procedure, router as trpcRouter } from "../trpc";
import { notFound } from "../utils/errors";
import { TRPCError } from "@trpc/server";

const dqsCluster = new Map<string, DqsServer>();
const queryRouter = trpcRouter({
  fetch: procedure
    .input(
      z.object({
        appId: z.string(),
        widgetId: z.string(),
        field: z.string(),
        // the last updated time of the widget so that to reload
        // data query if needed
        updatedAt: z.string(),
        params: z.record(z.any()),
      })
    )
    .query(async ({ ctx, input }): Promise<any> => {
      return await pipeRequestToDqs("QUERY", { ctx, input });
    }),
  mutate: procedure
    .input(
      z.object({
        appId: z.string(),
        widgetId: z.string(),
        field: z.string(),
        // the last updated time of the widget so that to reload
        // data query if needed
        updatedAt: z.string(),
        params: z.record(z.any()),
      })
    )
    .query(async ({ ctx, input }): Promise<any> => {
      return await pipeRequestToDqs("MUTATION", { ctx, input });
    }),
});

type Route = typeof queryRouter.fetch._def;
const pipeRequestToDqs = async (
  trigger: "QUERY" | "MUTATION",
  { input, ctx }: { ctx: Route["_ctx_out"]; input: Route["_input_out"] }
) => {
  const app = await ctx.repo.apps.fetchById(input.appId);
  if (!app) {
    return notFound();
  }
  const { workspaceId } = app;
  let server = dqsCluster.get(workspaceId);
  if (!server || !server.isAlive()) {
    server = await DqsCluster.startStreamServer(workspaceId);
    dqsCluster.set(workspaceId, server);
  }

  const response = await server.pipeRequest({
    url: "http://0.0.0.0/execWidgetQuery",
    method: "POST",
    headers: [["content-type", "application/json"]],
    body: {
      trigger,
      workspaceId,
      ...pick(input, "appId", "widgetId", "field", "params", "updatedAt"),
    },
  });

  // TODO(sagar): find a way to send response without converting to JSON
  const { result, error } = JSON.parse(
    String.fromCharCode.apply(null, response[2])
  );
  if (result) {
    return result.data;
  } else {
    throw new TRPCError({
      code: "BAD_REQUEST",
      message: error.message,
    });
  }
};

export { queryRouter };
