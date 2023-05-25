import { omit, merge } from "lodash-es";
import zod, { z } from "zod";
import { Widget } from "@arena/widgets";
import { badRequest, notFound } from "../utils/errors";
import { procedure, router as trpcRouter } from "../trpc";
import { dynamicSourceSchema } from "@arena/widgets/schema";
import { TEMPLATES } from "@arena/studio/templates";
import { MutationResponse } from "@arena/studio";
import { DbWidget } from "../repos/widget";

const widgetsRouter = trpcRouter({
  add: procedure
    .input(
      z.object({
        id: z.string(),
        appId: z.string(),
        description: z.string().optional(),
        parentId: z.string().nullable(),
        templateId: z.string(),
        position: z.object({
          /**
           * Id of the widget to place the new widget after
           */
          after: z.string().nullable(),
          /**
           * Id of the widget that existed the location of this widget
           * This is used to properly re-order the widgets
           */
          before: z.string().nullable(),
        }),
      })
    )
    .mutation(async ({ ctx, input }): Promise<MutationResponse> => {
      const { appId, parentId, templateId, position } = input;
      const app = await ctx.repo.apps.fetchById(appId);
      if (!app) {
        return notFound();
      }

      const templateMetadata = TEMPLATES[templateId].metadata;
      const defaultDataConfig = Object.fromEntries(
        Object.entries(templateMetadata.data).map(
          ([field, { title, source, default: config }]) => {
            return [field, { source, config }];
          }
        )
      ) as Widget["config"]["data"];

      /**
       * Need to check that position.before is valid and also update
       * the position.before's widget position
       */
      const widgetAfter = position.before
        ? await ctx.repo.widgets.fetchById(position.before)
        : null;

      if (
        widgetAfter &&
        widgetAfter?.config.layout?.position?.after !== position.after
      ) {
        return badRequest("Invalid widget position");
      }

      const newWidget = {
        id: input.id,
        name: templateMetadata.name,
        // TODO(sagar): generate slug
        slug: "",
        description: templateMetadata.description || "",
        parentId,
        appId: input.appId,
        templateId: input.templateId,
        config: {
          layout: {
            position: {
              after: input.position.after,
            },
          },
          data: defaultDataConfig,
          class: templateMetadata.class,
        },
      };

      const widget = {
        ...newWidget,
        // TODO(sagar): update
        createdBy: "sagar",
      };

      await ctx.repo.widgets.insert(widget);
      let updatedWidgetAfter = null;
      // update the widget after this widget's position
      if (widgetAfter) {
        updatedWidgetAfter = merge(widgetAfter, {
          config: {
            layout: {
              position: {
                after: widget.id,
              },
            },
          },
        });
        await ctx.repo.widgets.update(updatedWidgetAfter);
      }

      return {
        widgets: {
          created: [await setTemplateInfo(widget)],
          updated: updatedWidgetAfter
            ? [await setTemplateInfo(updatedWidgetAfter)]
            : [],
        },
      };
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
    .mutation<MutationResponse>(async ({ ctx, input }) => {
      const widget = await ctx.repo.widgets.fetchById(input.id);
      if (!widget) {
        notFound();
      }

      const updatedWidget = merge(widget, input);
      await ctx.repo.widgets.update(updatedWidget);

      return {
        widgets: {
          updated: [await setTemplateInfo(updatedWidget)],
        },
      };
    }),
  delete: procedure.input(z.any()).query(async () => {}),
});

const setTemplateInfo = async (widget: DbWidget): Promise<Widget> => {
  return merge(omit(widget, "templateId"), {
    template: {
      id: widget.templateId,
      name: "",
      url: "",
    },
  });
};

export { widgetsRouter };
