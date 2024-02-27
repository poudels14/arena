import { pick } from "lodash-es";
import { z } from "zod";
import { uniqueId } from "@portal/sdk/utils/uniqueId";
import { protectedProcedure } from "./procedure";
import { accessType } from "./repo/acl";

const addAcl = protectedProcedure
  .input(
    z.object({
      workspaceId: z.string(),
      userId: z.string(),
      access: accessType,
      app: z.object({
        id: z.string(),
        templateId: z.string(),
      }),
      metadata: z
        .object({
          table: z.string(),
          filter: z.string(),
          entities: z.array(
            z
              .object({
                id: z.string(),
              })
              .passthrough()
          ),
        })
        .passthrough(),
      resourceId: z.string().optional(),
    })
  )
  .mutate(async ({ ctx, body, errors }) => {
    const user = await ctx.repo.users.fetchById(body.userId);
    if (!user) {
      return errors.badRequest("user not found");
    }
    let workspace = await ctx.repo.workspaces.getWorkspaceById({
      id: body.workspaceId,
    });
    if (!workspace) {
      return errors.badRequest("workspace not found");
    }
    let app = await ctx.repo.apps.fetchById(body.app.id);
    if (!app) {
      return errors.badRequest("app not found");
    }
    const id = uniqueId(19);
    // TODO: check for duplicate access
    await ctx.repo.acl.addAccess({
      id,
      workspaceId: body.workspaceId,
      userId: body.userId,
      access: body.access,
      appId: body.app.id,
      appTemplateId: body.app.templateId,
      metadata: body.metadata,
      resourceId: body.resourceId,
    });
    return {
      id,
    };
  });

const listAcls = protectedProcedure.query(
  async ({ ctx, searchParams, errors }) => {
    if (!searchParams.appTemplateId) {
      return errors.badRequest("Missing required query param: `appTemplateId`");
    }
    const acls = await ctx.repo.acl.listAccess({
      userId: ctx.user!.id,
      workspaceId: searchParams.workspaceId,
      appId: searchParams.appId,
      appTemplateId: searchParams.appTemplateId,
    });

    return acls.map((acl) => {
      return pick(
        acl,
        "id",
        "workspaceId",
        "access",
        "appId",
        "metadata",
        "resourceId",
        "createdAt",
        "updatedAt"
      );
    });
  }
);

const archiveAcl = protectedProcedure.mutate(
  async ({ ctx, params, errors }) => {
    const acl = await ctx.repo.acl.getById(params.id);
    if (!acl) {
      return errors.notFound("acl doesn't exist");
    }
    await ctx.repo.acl.archiveAccess(acl.id);
    return { success: true };
  }
);

export { addAcl, listAcls, archiveAcl };
