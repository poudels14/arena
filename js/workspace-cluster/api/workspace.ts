import z from "zod";
import { protectedProcedure } from "./procedure";

const create = protectedProcedure
  .input(
    z.object({
      id: z.string().optional(),
      name: z.string().optional(),
    })
  )
  .mutate(async ({ ctx, body }) => {
    const workspace = await ctx.repo.workspaces.createWorkspace({
      ownerId: ctx.user!.id,
      id: body.id,
      name: body.name,
    });
    return workspace;
  });

export { create };
