import { merge, pick } from "lodash-es";
import { z } from "zod";
import { App, MutationResponse } from "@arena/studio";
import { procedure, router as trpcRouter } from "../trpc";
import { notFound } from "../utils/errors";
import { uniqueId } from "@arena/uikit/uniqueId";
// @ts-expect-error
import randomColor from "randomcolor";

const appsRouter = trpcRouter({
  add: procedure
    .input(
      z.object({
        workspaceId: z.string(),
        name: z.string(),
        description: z.string().optional(),
      })
    )
    .mutation(async ({ ctx, input }): Promise<MutationResponse> => {
      const newApp = await ctx.repo.apps.insert({
        id: uniqueId(),
        ...(input as Required<typeof input>),
        ownerId: "sagar",
        config: {
          ui: {
            thumbnail: createNewAppThumbnail(),
          },
        },
      });

      return {
        apps: {
          created: [pick(newApp, "id", "name", "description", "config")],
        },
      };
    }),
  list: procedure
    .input(
      z.object({
        workspaceId: z.string(),
      })
    )
    .query(
      async ({ ctx, input }): Promise<Omit<App, "widgets" | "resources">[]> => {
        const apps = await ctx.repo.apps.fetch({
          workspaceId: input.workspaceId,
          // TODO(sagar): dont hard code owner
          ownerId: "sagar",
        });
        return apps.map((app) => {
          return pick(app, "id", "name", "description", "config");
        });
      }
    ),
  get: procedure.input(z.string()).query<App>(async ({ ctx, input: appId }) => {
    const app = await ctx.repo.apps.fetchById(appId);
    if (!app) {
      return notFound("App not found");
    }
    const widgets = await ctx.repo.widgets.fetchByAppId(app.id);
    const resources = await ctx.repo.resources.fetchByWorkspaceId(
      app.workspaceId!
    );
    return merge(pick(app, "id", "name", "description", "config"), {
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
            merge(pick(app, "id", "name", "description", "config"), archivedAt),
          ],
        },
      };
    }),
});

const createNewAppThumbnail = () => {
  const gradientFrom = randomColor({
    luminosity: "light",
    format: "rgba",
    alpha: 0.8,
  });

  const gradientTo = randomColor({
    luminosity: "dark",
    format: "rgba",
    alpha: 0.8,
  });

  return {
    class: `from-[${gradientFrom.replaceAll(
      " ",
      ""
    )}] to-[${gradientTo.replaceAll(" ", "")}]`,
  };
};

export { appsRouter };
