import { protectedProcedure } from "./procedure";
import { listModels } from "./llm";

const list = protectedProcedure.query(async ({ req, ctx, params, errors }) => {
  if (!params.workspaceId) {
    return errors.badRequest("missing search param `workspaceId`");
  }
  const isWorkspaceMember = await ctx.repo.workspaces.isWorkspaceMember({
    userId: ctx.user.id,
    workspaaceId: params.workspaceId,
  });
  if (!isWorkspaceMember) {
    return errors.forbidden();
  }

  // @ts-expect-error
  const models: any[] = await listModels({
    ctx,
    searchParams: {
      workspaceId: params.workspaceId,
    },
    req,
    errors,
  });

  return {
    models,
  };
});

export { list };
