import { omit, merge } from "lodash-es";
import zod, { z } from "zod";
import { Widget } from "@arena/widgets";
import { notFound } from "../utils/errors";
import { procedure, router as trpcRouter } from "../trpc";
import { dbWidgetSchema } from "../repos/widget";
import { dynamicSourceSchema } from "@arena/widgets/schema";

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
        return notFound();
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
        id: z.string(),
        name: z.string().optional(),
        description: z.string().optional(),
        slug: z.string().optional(),
        // Note(sagar): rely on zod to ensure only dynamic data source is
        // updated and data source type can't be changed
        config: z
          .object({
            data: z.record(
              z.object({
                config: dynamicSourceSchema.shape.config,
              })
            ),
            class: z.string().optional(),
          })
          .optional(),
      })
    )
    .mutation(async ({ ctx, input }) => {
      const widget = await ctx.repo.widgets.fetchById(input.id);
      if (!widget) {
        notFound();
      }

      const updatedWidget = merge(widget, input);
      await ctx.repo.widgets.update(updatedWidget);
      return updatedWidget;
    }),
  delete: procedure.input(z.any()).query(async () => {}),
});

export { widgetsRouter };
