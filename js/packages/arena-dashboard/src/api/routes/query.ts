import { DqsServer, DqsCluster } from "@arena/runtime/dqs";
import { router as createRouter } from "@arena/runtime/server";
import { createContext } from "../context";

const queryRouter = createRouter({});
queryRouter.on(
  "GET",
  "/api/query/:appId/:widgetId/:field",
  async (req: any, res: any, params: any, store: any, searchParams: any) => {
    const ctx = await createContext({ req, resHeaders: new Headers() });
    await pipeRequestToDqs("QUERY", ctx, params, searchParams, res);
  }
);

queryRouter.on(
  "POST",
  "/api/query/:appId/:widgetId/:field",
  async (req: any, res: any, params: any, store: any, searchParams: any) => {
    const ctx = await createContext({ req, resHeaders: new Headers() });
    await pipeRequestToDqs("MUTATION", ctx, params, searchParams, res);
  }
);

const dqsCluster = new Map<string, DqsServer>();
const pipeRequestToDqs = async (
  trigger: "QUERY" | "MUTATION",
  ctx: any,
  params: Record<string, any>,
  searchParams: Record<string, any>,
  res: any
) => {
  const app = await ctx.repo.apps.fetchById(params.appId);
  if (!app) {
    res.sendResponse(
      new Response("Not found", {
        status: 404,
      })
    );
    return;
  }
  const workspaceId = app.workspaceId!;
  let server = dqsCluster.get(workspaceId);
  if (!server || !server.isAlive()) {
    server = await DqsCluster.startStreamServer(workspaceId);
    dqsCluster.set(workspaceId, server);
  }

  const [status, headers, body] = await server.pipeRequest({
    url: "http://0.0.0.0/execWidgetQuery",
    method: "POST",
    headers: [["content-type", "application/json"]],
    body: {
      trigger,
      workspaceId,
      appId: params.appId,
      widgetId: params.widgetId,
      field: params.field,
      params: JSON.parse(searchParams.params),
      updatedAt: searchParams.updatedAt,
    },
  });

  res.sendResponse(
    new Response(body, {
      status,
      headers: new Headers(headers),
    })
  );
};

export { queryRouter };
