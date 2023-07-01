import { DqsServer, DqsCluster } from "@arena/runtime/dqs";
import { createRouter, procedure } from "@arena/runtime/server";
import { Context } from "../context";

const p = procedure<Context>().use(async ({ ctx, params, errors, next }) => {
  if (!ctx.user || !(await ctx.acl.hasAppAccess(params.appId, "view-entity"))) {
    return errors.forbidden();
  }
  return next({});
});

const queryRouter = createRouter<Context>({
  prefix: "/w",
  routes: {
    "/:appId/widgets/:widgetId/api/:field": p
      .use(async ({ req, ctx, params, searchParams, next, errors }) => {
        if (req.method == "POST") {
          if (!(await ctx.acl.hasAppAccess(params.appId, "view-entity"))) {
            return errors.forbidden();
          }
          return await pipeRequestToDqs("MUTATION", ctx, params, searchParams);
        }
        return next({});
      })
      .query(async ({ ctx, params, searchParams }) => {
        return await pipeRequestToDqs("QUERY", ctx, params, searchParams);
      }),
  },
});

const dqsCluster = new Map<string, DqsServer>();
const pipeRequestToDqs = async (
  trigger: "QUERY" | "MUTATION",
  ctx: Context,
  params: Record<string, any>,
  searchParams: Record<string, any>
) => {
  const app = await ctx.repo.apps.fetchById(params.appId);
  if (!app) {
    return new Response("Not found", {
      status: 404,
    });
  }
  const workspaceId = app.workspaceId!;
  let server = dqsCluster.get(workspaceId);
  if (!server || !server.isAlive()) {
    server = await DqsCluster.startStreamServer(workspaceId);
    dqsCluster.set(workspaceId, server);
  }

  const { appId, widgetId, field } = params;
  const [status, headers, body] = await server.pipeRequest({
    url: `http://0.0.0.0/${appId}/widget/${widgetId}/api/${field}`,
    method: "POST",
    headers: [["content-type", "application/json"]],
    body: {
      trigger,
      workspaceId,
      appId,
      widgetId,
      field,
      props: JSON.parse(searchParams.props),
      updatedAt: searchParams.updatedAt,
    },
  });

  return new Response(body, {
    status,
    headers: new Headers(headers),
  });
};

export { queryRouter };
