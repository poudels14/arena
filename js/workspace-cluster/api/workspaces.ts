import z from "zod";
import { Context, protectedProcedure } from "./procedure";
import { addDatabase } from "./utils/database";
import slugify from "@sindresorhus/slugify";
import { uniqueId } from "@portal/sdk/utils/uniqueId";
import { pick } from "lodash-es";
import { listModels } from "./llm";
import { addApp } from "./utils/app";

const add = protectedProcedure
  .input(
    z.object({
      id: z.string().optional(),
      name: z.string().optional(),
    })
  )
  .mutate(async ({ ctx, body }) => {
    const repo = await ctx.repo.transaction();

    let workspaceId = slugify(body.id || uniqueId(), {
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

      const database = await addDatabase(repo, {
        id: workspaceId,
        workspaceId: workspace.id,
        user: "app",
      });

      await repo.commit();
      return {
        ...workspace,
        database,
      };
    } finally {
      await repo.release();
    }
  });

const list = protectedProcedure.query(async ({ ctx }) => {
  let workspaces = await ctx.repo.workspaces.listWorkspaces({
    userId: ctx.user!.id,
  });

  if (workspaces.length == 0) {
    await createNewWorkspace(ctx);
    workspaces = await ctx.repo.workspaces.listWorkspaces({
      userId: ctx.user!.id,
    });
  }

  return workspaces.map((workspace) => {
    return pick(workspace, "id", "name", "access");
  });
});

const get = protectedProcedure.query(async ({ req, ctx, params, errors }) => {
  const workspace = await ctx.repo.workspaces.getWorkspaceById({
    id: params.id,
  });

  if (!workspace) {
    return errors.notFound();
  }

  const hasAccess = await ctx.repo.workspaces.isWorkspaceMember({
    userId: ctx.user!.id,
    workspaaceId: workspace.id,
  });

  if (!hasAccess) {
    return errors.forbidden();
  }

  const apps = await ctx.repo.apps.listApps({
    workspaceId: workspace.id,
  });

  // @ts-expect-error
  const models: any[] = await listModels({
    ctx,
    searchParams: {
      workspaceId: workspace.id,
    },
    req,
    errors,
  }).then((res) => {
    return res instanceof Response ? [] : res;
  });

  return {
    ...pick(workspace, "id", "name", "access"),
    apps: apps.map((app) => {
      return pick(app, "id", "name", "slug", "description", "template");
    }),
    models: models.map((m) =>
      pick(
        m,
        "id",
        "name",
        "provider",
        "type",
        "modalities",
        "custom",
        "disabled",
        "requiresSubscription"
      )
    ),
  };
});

const createNewWorkspace = async (ctx: Context) => {
  const userId = ctx.user.id;
  const workspace = await ctx.repo.workspaces.createWorkspace({
    id: uniqueId(19),
    ownerId: userId,
    config: {
      runtime: {
        netPermissions: {
          // No restrictions by default
          restrictedUrls: [],
        },
      },
    },
  });

  const repo = await ctx.repo.transaction();
  try {
    const atlasAi = await ctx.repo.appTemplates.fetchById("atlasai");
    if (atlasAi) {
      await addApp(
        repo,
        { id: userId },
        {
          id: uniqueId(19),
          workspaceId: workspace.id,
          name: "Atlas AI",
          description: "An AI Assistant",
          template: {
            id: atlasAi.id,
            version: atlasAi.defaultVersion || "0.0.1",
          },
        }
      );
    }
    const portalDrive = await ctx.repo.appTemplates.fetchById("portal-drive");
    if (portalDrive) {
      await addApp(
        repo,
        { id: userId },
        {
          id: uniqueId(19),
          workspaceId: workspace.id,
          name: "Portal Drive",
          description: "Portal Drive",
          template: {
            id: portalDrive.id,
            version: portalDrive.defaultVersion || "0.0.1",
          },
        }
      );
    }
    await repo.commit();
  } finally {
    await repo.release();
  }
};

export { add, list, get };
