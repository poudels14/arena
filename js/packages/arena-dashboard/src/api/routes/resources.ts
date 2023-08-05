import { z } from "zod";
import { procedure, router as trpcRouter } from "../trpc";
import { MutationResponse } from "@arena/studio";
import { uniqueId } from "@arena/sdk/utils/uniqueId";
import { merge, snakeCase } from "lodash-es";
import { badRequest, notFound } from "../utils/errors";
import { checkResourceAccess, checkWorkspaceAccess } from "../middlewares/idor";

const resourceSchemaForClient = z.object({
  id: z.string(),
  workspaceId: z.string(),
  name: z.string(),
  description: z.string().optional().nullable(),
  type: z.string(),
  secret: z.boolean(),
  key: z.string().optional().nullable(),
  contextId: z.string().optional().nullable(),
  createdBy: z.string(),
  updatedAt: z.string(),
});

const resourceTypeSchemaForClient = z.object({
  id: z.string(),
  name: z.string(),
  description: z.string().optional().nullable(),
});

const resourcesRouter = trpcRouter({
  add: procedure
    .input(
      z.object({
        workspaceId: z.string(),
        name: z.string(),
        description: z.string().optional(),
        type: z.string(),

        key: z.string().optional(),
        value: z.any(),
        contextId: z.string().optional(),
      })
    )
    .use(checkWorkspaceAccess((input) => input.workspaceId, "member"))
    .mutation(async ({ ctx, input }): Promise<MutationResponse> => {
      // TODO(sp): validate workspace id

      const resourceType = (await ctx.repo.resources.fetchResourceTypes()).find(
        (t) => t.id == input.type
      );
      if (!resourceType) {
        return badRequest();
      }

      const newResource = await ctx.repo.resources.insert({
        id: uniqueId(16),
        ...(input as Required<typeof input>),
        // TODO(sp): if the key already exists, add sufix to make it unique
        key: snakeCase(input.key ? input.key : input.name).toUpperCase(),
        secret: resourceType.isSecret,
        createdBy: ctx.user!.id,
      });

      await ctx.repo.acl.addAccess({
        workspaceId: input.workspaceId,
        userId: ctx.user.id,
        access: "owner",
        resourceId: newResource.id,
      });

      return {
        resources: {
          created: [resourceSchemaForClient.parse(newResource)],
        },
      };
    }),
  listTypes: procedure.query(
    async ({
      ctx,
      input,
    }): Promise<z.infer<typeof resourceTypeSchemaForClient>[]> => {
      const resourceTypes = await ctx.repo.resources.fetchResourceTypes();

      return resourceTypes.map((type) =>
        resourceTypeSchemaForClient.parse(type)
      );
    }
  ),
  list: procedure
    .input(
      z.object({
        workspaceId: z.string(),
      })
    )
    .use(checkWorkspaceAccess((input) => input.workspaceId, "member"))
    .query(
      async ({
        ctx,
        input,
      }): Promise<z.infer<typeof resourceSchemaForClient>[]> => {
        // TODO(sp): validate workspace id
        const resources = await ctx.acl.filterResourcesByAccess(
          await ctx.repo.resources.fetch({
            workspaceId: input.workspaceId,
          }),
          "view-entity"
        );

        return resources.map((resource) =>
          resourceSchemaForClient.parse(resource)
        );
      }
    ),
  archive: procedure
    .input(
      z.object({
        id: z.string(),
        workspaceId: z.string(),
      })
    )
    .use(checkResourceAccess((input) => input.id, "admin"))
    .mutation(async ({ ctx, input }): Promise<MutationResponse> => {
      const queryToArchive = await ctx.repo.resources.fetchById(input.id);
      if (!queryToArchive || queryToArchive.workspaceId != input.workspaceId) {
        return notFound();
      }

      const { archivedAt } = await ctx.repo.resources.archiveById(input.id);

      return {
        resources: {
          deleted: [
            resourceSchemaForClient.parse(
              merge(queryToArchive, {
                archivedAt,
              })
            ),
          ],
        },
      };
    }),
});

export { resourcesRouter };
