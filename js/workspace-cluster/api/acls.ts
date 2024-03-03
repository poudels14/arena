import { pick, merge } from "lodash-es";
import { z } from "zod";
import { uniqueId } from "@portal/sdk/utils/uniqueId";
import { protectedProcedure } from "./procedure";
import { aclCommand } from "./repo/acl";

const addAcl = protectedProcedure
  .input(
    z.object({
      userId: z.string(),
      accessGroup: z.string(),
      app: z.object({
        id: z.string(),
      }),
      metadata: z
        .object({
          filters: z.array(
            z.object({
              command: aclCommand,
              table: z.string(),
              condition: z.string(),
            })
          ),
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
    if (body.userId != "public") {
      const user = await ctx.repo.users.fetchById(body.userId);
      if (!user) {
        return errors.badRequest("user not found");
      }
    }

    let app = await ctx.repo.apps.fetchById(body.app.id);
    if (!app) {
      return errors.badRequest("app not found");
    }

    if (app.ownerId != ctx.user!.id) {
      return errors.forbidden();
    }

    const id = uniqueId(19);
    // TODO: check for duplicate access
    await ctx.repo.acl.addAccess({
      id,
      workspaceId: app.workspaceId,
      userId: body.userId,
      accessGroup: body.accessGroup,
      appId: body.app.id,
      appTemplateId: app.template?.id!,
      metadata: body.metadata,
      resourceId: body.resourceId,
    });

    // reboot app such that new acls are applied
    await ctx.repo.appDeployments.reboot({
      appId: body.app.id,
    });

    return {
      id,
    };
  });

const listUserAcls = protectedProcedure.query(
  async ({ ctx, searchParams, errors }) => {
    if (!searchParams.appTemplateId) {
      return errors.badRequest("Missing required query param: `appTemplateId`");
    }
    // if appId is given, userId is optional but requesting user should be owner
    // of the app id. if user making the request isn't the owner, only the acl of
    // the requesting user is returned
    // if appId isn't given, id of the user making the request is used as userId
    let userId = searchParams.userId;
    if (searchParams.appId) {
      const app = await ctx.repo.apps.fetchById(searchParams.appId);
      if (!app) {
        return errors.notFound("App not found");
      }
      // if user isn't the owner, only return user's acl
      if (app.ownerId != ctx.user!.id) {
        userId = ctx.user!.id;
      }
    }
    const acls = await ctx.repo.acl.listAccess({
      userId,
      workspaceId: searchParams.workspaceId,
      appId: searchParams.appId,
      appTemplateId: searchParams.appTemplateId,
    });

    return acls.map((acl) => {
      return merge(
        pick(
          acl,
          "id",
          "workspaceId",
          "userId",
          "accessGroup",
          "appId",
          "resourceId",
          "updatedAt"
        ),
        {
          metadata: pick(acl.metadata, "entities"),
        }
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
    // reboot app such that updated acl list is applied
    await ctx.repo.appDeployments.reboot({
      appId: acl.appId!,
    });
    return { success: true };
  }
);

export { addAcl, listUserAcls, archiveAcl };
