import { omit, merge, compact, pick } from "lodash-es";
import zod, { z } from "zod";
import { Widget } from "@arena/widgets";
import camelCase from "camelcase";
import {
  DataSource,
  Template,
  dynamicSourceSchema,
  userInputSourceSchema,
  widgetConfigSourceSchema,
} from "@arena/widgets/schema";
import { TEMPLATES } from "@arena/studio/templates";
import { MutationResponse } from "@arena/studio";
import { transpileServerFunction } from "@arena/cloud/query";
import { DbWidget, createRepo } from "../repos/widget";
import { badRequest, notFound } from "../utils/errors";
import { procedure, router as trpcRouter } from "../trpc";

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
    config: z.union([
      // Note(sp): should be careful with zod validation; z.any() type should
      // come before other types, otherwise casting could result in data of
      // type any() being striped out
      widgetConfigSourceSchema.shape.config,
      dynamicSourceSchema.shape.config,
      userInputSourceSchema.shape.config,
    ]),
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
          ([field, { source, default: config }]: any) => {
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
          data: await withDefaultSourceLoaderConfig(
            input.templateId,
            defaultDataConfig
          ),
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
        const template = TEMPLATES[widget.templateId].metadata;
        const updatableFields = Object.entries(template.data)
          .filter(([f, c]) =>
            ["dynamic", "userinput", "config"].includes(c.source)
          )
          .map(([f]) => f);

        const dataConfigPatch = pick(config.data, updatableFields);

        merge(
          widget.config.data,
          await withDefaultSourceLoaderConfig(
            widget.templateId,
            // @ts-expect-error
            dataConfigPatch
          )
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

const withDefaultSourceLoaderConfig = async (
  templateId: DbWidget["templateId"],
  dataConfig: Record<string, DataSource<any>>
) => {
  const templateMetadata = TEMPLATES[templateId]
    .metadata as Template.Metadata<any>;
  const { fromEntries, entries } = Object;
  return fromEntries(
    await Promise.all(
      entries(dataConfig).map(async ([field, fieldConfig]: any) => {
        const templateFieldConfig = templateMetadata.data[field];
        const { source } = templateFieldConfig;
        if (source != "dynamic" && source != "template") {
          return [field, fieldConfig];
        }
        const updatedFieldConfig = {
          ...fieldConfig,
          config: {
            ...fieldConfig.config,
            value:
              fieldConfig.config.value ||
              getDefaultValueForLoader(
                fieldConfig.config.loader,
                (templateMetadata.data[field] as any).preview
              ),
          },
        };

        await validateDataSource(updatedFieldConfig);
        return [field, updatedFieldConfig];
      })
    )
  );
};

const validateDataSource = async (
  dataSource: DataSource.Dynamic | DataSource.Template
) => {
  const { config } = dataSource;
  if (config.loader == "@arena/server-function") {
    const transpiled = await transpileServerFunction(config.value);
    config.metatada = {
      ...transpiled,
    };
  }
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
      return (
        "function execute(app, widgets) {\n" +
        "\treturn " +
        dataJson +
        "\n" +
        "}"
      );
    case "@arena/sql/postgres":
      return "SELECT 1;";
    case "@arena/server-function":
      // TODO(sagar): return default preview value
      return (
        "export default function({ env }) {\n" +
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
