import { merge, pick } from "lodash-es";
import { z } from "zod";
import { App, MutationResponse } from "@arena/studio";
import { uniqueId } from "@arena/sdk/utils/uniqueId";
// @ts-expect-error
import randomColor from "randomcolor";
import { procedure, router as trpcRouter } from "../trpc";
import { notFound } from "../utils/errors";
import { BUILTIN_APPS } from "~/BUILTIN_APPS";
import { checkAppAccess, checkWorkspaceAccess } from "../middlewares/idor";

const appsRouter = trpcRouter({
  add: procedure
    .input(
      z.object({
        workspaceId: z.string(),
        name: z.string(),
        description: z.string().optional(),
        template: z
          .object({
            id: z.string(),
            version: z.string(),
          })
          .optional(),
      })
    )
    .use(checkWorkspaceAccess((input) => input.workspaceId, "member"))
    .mutation(async ({ ctx, input }): Promise<MutationResponse> => {
      const newApp = await ctx.repo.apps.insert({
        id: uniqueId(),
        ...(input as Required<typeof input>),
        createdBy: ctx.user!.id,
        config: {
          ui: {
            thumbnail: createNewAppThumbnail(),
          },
        },
      });

      await ctx.repo.acl.addAccess({
        workspaceId: input.workspaceId,
        userId: ctx.user.id,
        access: "owner",
        appId: newApp.id,
      });

      return {
        apps: {
          created: [
            merge(pick(newApp, "id", "name", "description", "config"), {
              template: getTemplateManifest(newApp.template),
            }),
          ],
        },
      };
    }),
  list: procedure
    .input(
      z.object({
        workspaceId: z.string(),
      })
    )
    .use(checkWorkspaceAccess((input) => input.workspaceId, "member"))
    .query(
      async ({
        ctx,
        input,
      }): Promise<Omit<App, "widgets" | "resources" | "template">[]> => {
        const apps = await ctx.acl.filterAppsByAccess(
          await ctx.repo.apps.listApps({
            workspaceId: input.workspaceId,
          }),
          "view-entity"
        );

        return apps.map((app) => {
          return pick(app, "id", "name", "description", "config");
        });
      }
    ),
  get: procedure
    .input(z.string())
    .use(checkAppAccess((input) => input, "view-entity"))
    .query<App>(async ({ ctx, input: appId }) => {
      const app = await ctx.repo.apps.fetchById(appId);
      if (!app) {
        return notFound("App not found");
      }
      const widgets = await ctx.repo.widgets.fetchByAppId(app.id);
      const resources = await ctx.acl.filterResourcesByAccess(
        await ctx.repo.resources.fetch({
          workspaceId: app.workspaceId!,
        }),
        "view-entity"
      );

      return merge(pick(app, "id", "name", "description", "config"), {
        template: getTemplateManifest(app.template),
        widgets: widgets.reduce((widgetsById, widget) => {
          widgetsById[widget.id] = {
            ...widget,
            template: {
              id: widget.templateId,
              // TODO
              name: "",
              url: "",
            },
          };
          return widgetsById;
        }, {} as App["widgets"]),
        resources: resources.reduce((resourcesById, resource) => {
          resourcesById[resource.id!] = pick(
            resource as App["resources"][""],
            "id",
            "name",
            "description",
            "type"
          );
          return resourcesById;
        }, {} as App["resources"]),
      });
    }),
  update: procedure
    .input(
      z.object({
        name: z.string().optional(),
        description: z.string().optional(),
        widgetId: z.string().optional(),
      })
    )
    .mutation(async ({ input }) => {
      // TODO: update app
    }),
  archive: procedure
    .input(
      z.object({
        workspaceId: z.string(),
        id: z.string(),
      })
    )
    .use(checkAppAccess((input) => input.id, "admin"))
    .mutation(async ({ ctx, input }): Promise<MutationResponse> => {
      // TODO: check IDOR
      const app = await ctx.repo.apps.fetchById(input.id);
      if (!app || app.workspaceId != input.workspaceId) {
        return notFound("App not found");
      }

      const { archivedAt } = await ctx.repo.apps.archiveById(input.id);
      return {
        apps: {
          deleted: [
            merge(pick(app, "id", "name", "description", "config"), {
              template: getTemplateManifest(app.template),
              archivedAt,
            }),
          ],
        },
      };
    }),
});

const createNewAppThumbnail = () => {
  const gradientFrom = randomColor({
    luminosity: "light",
    format: "rgba",
    alpha: 0.3,
  });

  const gradientTo = randomColor({
    luminosity: "dark",
    format: "rgba",
    alpha: 0.5,
  });

  return {
    class: `from-[${gradientFrom.replaceAll(
      " ",
      ""
    )}] to-[${gradientTo.replaceAll(" ", "")}]`,
  };
};

const getTemplateManifest = (
  template: { id: string; version: string } | null | undefined
) => {
  if (!template) {
    return null;
  }
  return (
    BUILTIN_APPS.find(
      (bt) => bt.id == template.id && bt.version == template.version
    ) || null
  );
};

export { appsRouter };
