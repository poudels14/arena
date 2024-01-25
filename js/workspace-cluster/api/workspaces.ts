import z from "zod";
import { protectedProcedure } from "./procedure";
import { addDatabase } from "./utils/database";
import slugify from "@sindresorhus/slugify";
import { uniqueId } from "@portal/sdk/utils/uniqueId";
import { pick } from "lodash-es";

const add = protectedProcedure
  .input(
    z.object({
      id: z.string().optional(),
      name: z.string().optional(),
    })
  )
  .mutate(async ({ ctx, body }) => {
    const repo = await ctx.repo.transaction();

    let workspaceId = slugify(body.id || uniqueId(18), {
      separator: "_",
      decamelize: false,
    });
    if (!workspaceId.startsWith("w_")) {
      workspaceId = "w_" + workspaceId;
    }

    let workspace;
    try {
      workspace = await repo.workspaces.createWorkspace({
        ownerId: ctx.user!.id,
        id: workspaceId,
        name: body.name,
      });
    } catch (e) {
      console.error(e);
      throw e;
    }

    const database = await addDatabase(repo, {
      id: workspaceId,
      workspaceId: workspace.id,
      user: "app",
    });

    await repo.commit();
    await repo.release();
    return {
      ...workspace,
      database,
    };
  });

const list = protectedProcedure.query(async ({ ctx }) => {
  const workspaces = await ctx.repo.workspaces.listWorkspaces({
    userId: ctx.user!.id,
  });
  return workspaces.map((workspace) => {
    return pick(workspace, "id", "name", "access");
  });
});

const get = protectedProcedure.query(async ({ ctx, params, errors }) => {
  const workspace = await ctx.repo.workspaces.getWorkspaceById({
    id: params.id,
    userId: ctx.user!.id,
  });

  if (!workspace) {
    return errors.notFound();
  }

  const apps = await ctx.repo.apps.listApps({
    workspaceId: workspace.id,
  });

  return {
    ...pick(workspace, "id", "name", "access"),
    apps: apps.map((app) => {
      return pick(app, "id", "name", "slug", "description", "template");
    }),
  };
});

export { add, list, get };
