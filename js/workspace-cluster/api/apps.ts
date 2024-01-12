import { merge, pick } from "lodash-es";
import { z } from "zod";
import slugify from "@sindresorhus/slugify";
import { uniqueId } from "@portal/sdk/utils/uniqueId";
import { protectedProcedure } from "./procedure";
import { addDatabase } from "./utils/database";

const add = protectedProcedure
  .input(
    z.object({
      id: z.string().optional(),
      workspaceId: z.string(),
      name: z.string(),
      description: z.string().optional(),
      template: z.object({
        id: z.string(),
        version: z.string(),
      }),
    })
  )
  .mutate(async ({ ctx, body, errors }) => {
    const workspace = await ctx.repo.workspaces.getWorkspaceById(
      body.workspaceId
    );
    if (!workspace) {
      return errors.badRequest("Invalid workspace id");
    }

    const repo = await ctx.repo.transaction();
    const appId = body.id || uniqueId(19);

    const database = await addDatabase(repo, {
      id: workspace.id,
      workspaceId: workspace.id,
      appId,
      user: "app",
    });

    const newApp = await repo.apps.insert({
      id: appId,
      workspaceId: body.workspaceId,
      name: body.name,
      slug: slugify(body.name, {
        separator: "_",
      }),
      description: body.description || "",
      template: body.template,
      createdBy: ctx.user!.id,
      config: {},
    });

    await repo.commit();
    await repo.release();
    return {
      ...pick(
        newApp,
        "id",
        "name",
        "slug",
        "description",
        "workspaceId",
        "config",
        "template"
      ),
      database,
    };
  });

const list = protectedProcedure.query(async ({ ctx, searchParams, errors }) => {
  if (!searchParams.workspaceId) {
    return errors.badRequest("Missing query param: `workspaceId`");
  }
  const apps = await ctx.repo.apps.listApps({
    workspaceId: searchParams.workspaceId,
  });
  return apps.map((app) => {
    return pick(app, "id", "name", "slug", "description", "config", "template");
  });
});

const archive = protectedProcedure
  .input(
    z.object({
      workspaceId: z.string(),
      id: z.string(),
    })
  )
  .mutate(async ({ ctx, body, errors }) => {
    // TODO: check IDOR
    const app = await ctx.repo.apps.fetchById(body.id);
    if (!app || app.workspaceId != body.workspaceId) {
      return errors.notFound("App not found");
    }

    const { archivedAt } = await ctx.repo.apps.archiveById(body.id);
    return {
      apps: {
        deleted: [
          merge(pick(app, "id", "name", "slug", "description", "config"), {
            archivedAt,
          }),
        ],
      },
    };
  });

export { add, list, archive };
