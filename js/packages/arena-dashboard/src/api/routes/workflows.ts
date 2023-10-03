import { createRouter, procedure } from "@arena/runtime/server";
import { uniqueId } from "@arena/sdk/utils/uniqueId";
import { Status } from "@arena/sdk/plugins/workflow";
import * as jwt from "@arena/cloud/jwt";
import { Context } from "../context";
import { z } from "zod";

const workflowTemplateSchema = z.object({
  plugin: z.object({
    id: z.string(),
    version: z.string(),
  }),
  slug: z.string(),
});

const p = procedure<Context>();
const workflowsRouter = createRouter<any>({
  prefix: "/workflows",
  routes: {
    "/create": p.mutate(async ({ ctx, req, errors }) => {
      console.log("HEADERS =", req.headers);

      let triggeredBy;
      try {
        const { payload } = jwt.verify(
          req.headers.get("x-arena-authorization") || "",
          "HS256",
          process.env.JWT_SIGNINIG_SECRET
        );
        triggeredBy = payload;
      } catch (e) {
        return errors.forbidden();
      }

      if (!triggeredBy.app) {
        return errors.badRequest("Only app can run a workflow");
      }

      const app = await ctx.repo.apps.fetchById(triggeredBy.app.id);
      if (!app) {
        return errors.forbidden();
      }

      const { template: rawTemplate, input } = await req.json();
      const result = workflowTemplateSchema.safeParse(rawTemplate);
      if (!result.success) {
        return errors.badRequest("Invalid workflow template");
      }

      // TODO(sagar): validate plugin template id/version
      const { data: template } = result;

      const id = "wfr-" + uniqueId(15);
      await ctx.repo.workflowRuns.insertWorkflowRun({
        id,
        workspaceId: app.workspaceId,
        parentAppId: app.id,
        triggeredBy: {
          user: {
            id: ctx.user.id,
          },
          app: {
            id: app.id,
          },
        },
        config: {},
        state: {
          input,
        },
        status: Status.CREATED,
        template,
        triggeredAt: new Date(),
        lastHeartbeatAt: null,
      });

      return { id };
    }),
  },
});

export { workflowsRouter };
