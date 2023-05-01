import { pick } from "lodash-es";
import zod, { z } from "zod";
import { App } from "@arena/studio";
import { procedure, router as trpcRouter } from "../trpc";
import { notFound } from "../utils/errors";

const appsRouter = trpcRouter({
  getApp: procedure
    .input(zod.string())
    .query(async ({ ctx, input: appId }): Promise<App> => {
      const app = await ctx.repo.apps.fetchById(appId);
      if (!app) {
        return notFound("App not found");
      }
      const widgets = await ctx.repo.widgets.fetchByAppId(app.id);
      return Object.assign(
        {
          widgets: widgets.map((w) => ({
            ...w,
            template: {
              id: w.templateId,
              // TODO
              name: "",
              url: "",
            },
          })),
        },
        pick(app, "id", "name", "description")
      );
    }),
  updateApp: procedure
    .input(
      zod.object({
        name: z.string().optional(),
        description: z.string().optional(),
        widgetId: z.string().optional(),
      })
    )
    .mutation(async ({ input }) => {
      // TODO: update app
    }),
});

export { appsRouter };
