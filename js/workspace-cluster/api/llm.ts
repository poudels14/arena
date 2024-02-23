import { protectedProcedure } from "./procedure";

const list = protectedProcedure.query(async ({ ctx, searchParams, errors }) => {
  if (!searchParams.workspaceId) {
    return errors.badRequest("Missing query param: `workspaceId`");
  }
  const workspace = await ctx.repo.workspaces.getWorkspaceById({
    id: searchParams.workspaceId,
  });
  if (!workspace) {
    return errors.badRequest("Invalid workspace id");
  }

  return errors.badRequest("not implemented");
});

export { list };
