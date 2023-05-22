import { omit } from "lodash-es";
import zod, { z } from "zod";
import { Widget } from "@arena/appkit/widget";
import { notFound } from "../utils/errors";
import { procedure, router as trpcRouter } from "../trpc";
import { dbWidgetSchema } from "../repos/widget";

const addWidgetSchema = dbWidgetSchema.omit({
  createdBy: true,
  archivedAt: true,
});

const widgetsRouter = trpcRouter({
  add: procedure
    .input(addWidgetSchema)
    .mutation(async ({ ctx, input }): Promise<Widget> => {
      const app = await ctx.repo.apps.fetchById(input.appId);
      if (!app) {
        return notFound("App not found");
      }

      const widget = {
        ...input,
        // TODO(sagar): update
        createdBy: "sagar",
      };

      await ctx.repo.widgets.insert(widget);
      return Object.assign(omit(widget, "templateId"), {
        template: {
          id: widget.templateId,
          name: "",
          url: "",
        },
      });
    }),
  update: procedure
    .input(
      zod.object({
        name: z.string().optional(),
        description: z.string().optional(),
        widgetId: z.string().optional(),
        config: z.any(),
      })
    )
    .mutation(async ({ input }) => {
      throw new Error("Not implemented");
    }),
  delete: procedure.input(z.any()).query(async () => {}),
});

export { widgetsRouter };
