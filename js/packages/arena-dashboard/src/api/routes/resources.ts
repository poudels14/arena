import { z } from "zod";
import { procedure, router as trpcRouter } from "../trpc";
import { MutationResponse } from "@arena/studio";
import { uniqueId } from "@arena/uikit/uniqueId";
import { merge } from "lodash-es";
import { notFound } from "../utils/errors";

const resourceSchemaForClient = z.object({
  id: z.string(),
  workspaceId: z.string(),
  name: z.string(),
  description: z.string().optional().nullable(),
  type: z.enum(["@arena/sql/postgres", "env", "config"]),
  secret: z.boolean(),
  key: z.string().optional().nullable(),
  contextId: z.string().optional().nullable(),
  createdBy: z.string(),
  updatedAt: z.string(),
});

const resourcesRouter = trpcRouter({
  add: procedure
    .input(
      z.object({
        workspaceId: z.string(),
        name: z.string(),
        description: z.string().optional(),
        type: z.enum(["@arena/sql/postgres", "env", "config"]),

        key: z.string().optional(),
        value: z.any(),
        contextId: z.string().optional(),
      })
    )
    .mutation(async ({ ctx, input }): Promise<MutationResponse> => {
      // TODO(sp): validate workspace id

      const isSecret = ["@arena/sql/postgres"].includes(input.type);
      const newResource = await ctx.repo.resources.insert({
        id: uniqueId(),
        ...(input as Required<typeof input>),
        secret: isSecret,
        createdBy: "sagar",
      });

      return {
        resources: {
          created: [resourceSchemaForClient.parse(newResource)],
        },
      };
    }),
  list: procedure
    .input(
      z.object({
        workspaceId: z.string(),
      })
    )
    .query(
      async ({
        ctx,
        input,
      }): Promise<z.infer<typeof resourceSchemaForClient>[]> => {
        // TODO(sp): validate workspace id
        const resources = await ctx.repo.resources.fetchByWorkspaceId(
          input.workspaceId
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
