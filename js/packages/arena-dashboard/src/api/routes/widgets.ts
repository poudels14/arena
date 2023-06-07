import { omit, merge, compact } from "lodash-es";
import zod, { z } from "zod";
import { Widget } from "@arena/widgets";
import camelCase from "camelcase";
import { badRequest, notFound } from "../utils/errors";
import { procedure, router as trpcRouter } from "../trpc";
import {
  DataSource,
  Template,
  dynamicSourceSchema,
} from "@arena/widgets/schema";
import { TEMPLATES } from "@arena/studio/templates";
import { MutationResponse } from "@arena/studio";
import { DbWidget, createRepo } from "../repos/widget";

type DbRepo = { widgets: ReturnType<typeof createRepo> };

const layoutUpdateSchema = z.object({
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
});

const dataUpdateSchema = z.record(
  z.object({
    config: dynamicSourceSchema.shape.config,
  })
);

const widgetsRouter = trpcRouter({
  add: procedure
    .input(
      z.object({
        id: z.string(),
        appId: z.string(),
        description: z.string().optional(),
        parentId: z.string().nullable(),
        templateId: z.string(),
        config: z.object({
          layout: layoutUpdateSchema,
          data: dataUpdateSchema.optional(),
          class: z.string().optional(),
        }),
      })
    )
    .mutation(async ({ ctx, input }): Promise<MutationResponse> => {
      const { appId, parentId, templateId, config } = input;
      const { layout } = config;
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
      let widgetAfter = await getNextWidgetInLinkedList(
        ctx.repo,
        layout.position
      );

      const newWidget = {
        id: input.id,
        name: templateMetadata.name,
        slug: camelCase(templateMetadata.name, { pascalCase: true }),
        description: templateMetadata.description || "",
        parentId,
        appId: input.appId,
        templateId: input.templateId,
        config: {
          layout: {
            position: {
              after: layout.position.after,
            },
          },
          // @ts-expect-error
          data: withDefaultSourceConfig(input.templateId, defaultDataConfig),
          config: z.any(),
          class: templateMetadata.class,
        },
      };

      const widget = await ctx.repo.widgets.insert({
        ...newWidget,
        // TODO(sagar): update
        createdBy: "sagar",
      });
      if (widgetAfter) {
        widgetAfter.config.layout.position = {
          after: widget.id,
        };
        widgetAfter = await ctx.repo.widgets.update(widgetAfter);
      }

      return {
        widgets: {
          created: await setTemplateInfo([widget]),
          updated: widgetAfter ? await setTemplateInfo([widgetAfter]) : [],
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
            layout: layoutUpdateSchema.optional(),
            data: dataUpdateSchema.optional(),
            config: z.any(),
            class: z.string().optional(),
          })
          .optional(),
      })
    )
    .mutation<MutationResponse>(async ({ ctx, input }) => {
      const widget = await ctx.repo.widgets.fetchById(input.id);
      if (!widget) {
        return notFound();
      }

      const { config, ...rest } = input;
      merge(widget, rest);

      merge(widget, rest);
      if (config?.data) {
        merge(
          widget.config.data,
          // @ts-expect-error
          withDefaultSourceConfig(widget.templateId, config.data)
        );
      }

      let widgetAfter;
      if (config?.layout) {
        if (
          (widgetAfter = await getNextWidgetInLinkedList(
            ctx.repo,
            config.layout.position
          ))
        ) {
          widgetAfter = await ctx.repo.widgets.update(
            merge(widgetAfter, ["config", "layout", "position"], {
              after: widget,
            })
          );
        }
      }

      if (config?.class) {
        widget.config.class = [...new Set(config.class.split(" "))]
          .map((c) => c.trim())
          .filter((c) => c.length > 0)
          .join(" ");
      }

      if (config?.config) {
        widget.config.config = config?.config;
      }

      const updatedWidgets = compact([
        await ctx.repo.widgets.update(widget),
        widgetAfter ? await ctx.repo.widgets.update(widgetAfter) : null,
      ]);
      return {
        widgets: {
          updated: await setTemplateInfo(updatedWidgets),
        },
      };
    }),
  delete: procedure
    .input(
      zod.object({
        id: z.string(),
        config: z.object({
          layout: layoutUpdateSchema,
        }),
      })
    )
    .mutation(async ({ ctx, input }) => {
      const { config } = input;
      const { position } = config.layout;
      const widgetToDelete = await ctx.repo.widgets.fetchById(input.id);
      if (!widgetToDelete) {
        return notFound();
      }

      let widgetAfter = await getNextWidgetInLinkedList(ctx.repo, {
        before: position.before,
        after: input.id,
      });
      if (!widgetAfter && position.before) {
        return badRequest();
      }

      if (widgetAfter) {
        widgetAfter.config.layout.position.after =
          widgetToDelete.config.layout.position.after;
        widgetAfter = await ctx.repo.widgets.update(widgetAfter);
      }

      // when deleting a widget, delete all of it's decendants
      const widgetsToDelete = [widgetToDelete.id];
      const ancestors: string[] = [];
      let parentId: string | undefined = widgetToDelete.id;
      while (parentId) {
        const childrenIds = await ctx.repo.widgets.getChildenWidgetsIds(
          parentId
        );
        childrenIds.forEach((child) => {
          widgetsToDelete.push(child);
          ancestors.push(child);
        });
        parentId = ancestors.pop();
      }

      await ctx.repo.widgets.archive(widgetsToDelete);
      return {
        widgets: {
          updated: await setTemplateInfo(compact([widgetAfter])),
          deleted: await setTemplateInfo([widgetToDelete]),
        },
      };
    }),
});

const setTemplateInfo = async (widgets: DbWidget[]): Promise<Widget[]> => {
  return widgets.map((widget) =>
    merge(omit(widget, "templateId"), {
      template: {
        id: widget.templateId,
        name: "",
        url: "",
      },
    })
  );
};

const getNextWidgetInLinkedList = async (
  repo: DbRepo,
  position: {
    before: string | null;
    after: string | null;
  }
) => {
  /**
   * Need to check that position.before is valid and also update
   * the position.before's widget position
   */
  const nextWidget = position.before
    ? await repo.widgets.fetchById(position.before)
    : null;

  if (
    nextWidget &&
    nextWidget?.config.layout.position.after !== position.after
  ) {
    return badRequest("Invalid widget position");
  }
  return nextWidget;
};

const withDefaultSourceConfig = (
  templateId: DbWidget["templateId"],
  dataConfig: Record<string, DataSource.Dynamic>
) => {
  const templateMetadata = TEMPLATES[templateId]
    .metadata as Template.Metadata<any>;
  const { fromEntries, entries } = Object;
  return fromEntries(
    entries(dataConfig).map(([field, fieldConfig]) => {
      return [
        field,
        {
          ...fieldConfig,
          config: {
            ...fieldConfig.config,
            value:
              fieldConfig.config.value ||
              getDefaultValueForLoader(
                fieldConfig.config.loader,
                templateMetadata.data[field].preview
              ),
          },
        },
      ];
    })
  );
};

const getDefaultValueForLoader = (
  loader: DataSource.Dynamic["config"]["loader"],
  previewData: any
) => {
  const dataJson = JSON.stringify(previewData, null, 4)
    .split("\n")
    .reduce((agg, line, index, lines) => {
      const isLast = index == lines.length - 1;
      agg += "\t" + line;
      return isLast ? agg + ";" : agg + "\n";
    }, "");

  switch (loader) {
    case "@client/json":
      return previewData;
    case "@client/js":
      return "function query() {\n" + "\treturn " + dataJson + "\n" + "}";
    case "@arena/sql/postgres":
      return "SELECT 1;";
    case "@arena/server-function":
      // TODO(sagar): return default preview value
      return (
        "export default function({ env, params }) {\n" +
        "\treturn " +
        dataJson +
        "\n" +
        "}"
      );
    default:
      return badRequest("Unsupported data source");
  }
};

export { widgetsRouter };
