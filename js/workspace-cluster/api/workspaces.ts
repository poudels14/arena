import z from "zod";
import { protectedProcedure } from "./procedure";
import { addDatabase } from "./utils/database";
import slugify from "@sindresorhus/slugify";
import { uniqueId } from "@portal/sdk/utils/uniqueId";

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
    const workspace = await repo.workspaces.createWorkspace({
      ownerId: ctx.user!.id,
      id: workspaceId,
      name: body.name,
    });

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

const list = protectedProcedure.query(async ({ ctx, env, searchParams }) => {
  const workspaces = await ctx.repo.workspaces.listWorkspaces({
    userId: searchParams.userId,
  });
  return workspaces;
});

export { add, list };
