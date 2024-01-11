import z from "zod";
import { protectedProcedure } from "./procedure";
import { uniqueId } from "@portal/sdk/utils/uniqueId";
import { pick } from "lodash-es";

const addCluster = protectedProcedure
  .input(
    z.object({
      id: z.string().optional(),
      host: z.string(),
      port: z.number(),
      capacity: z.number(),
      credentials: z.object({
        adminUser: z.string(),
        adminPassword: z.string(),
      }),
    })
  )
  .mutate(async ({ ctx, body }) => {
    const id = body.id || uniqueId();
    await ctx.repo.dbClusters.add({
      id,
      host: body.host,
      port: body.port,
      capacity: body.capacity,
      credentials: body.credentials,
    });

    return { id };
  });

const listClusters = protectedProcedure.query(async ({ ctx }) => {
  const clusters = await ctx.repo.dbClusters.list();
  return clusters.map((cluster) => {
    return pick(cluster, "id", "host", "port", "capacity", "usage");
  });
});

const deleteCluster = protectedProcedure
  .input(
    z.object({
      id: z.string(),
    })
  )
  .mutate(async ({ ctx, body }) => {
    await ctx.repo.dbClusters.delete({ id: body.id });
    return { id: body.id };
  });

const list = protectedProcedure.query(async ({ ctx, searchParams, errors }) => {
  if (!searchParams.workspaceId) {
    return errors.badRequest("query parameter `workspaceId` missing");
  }

  const databases = await ctx.repo.databases.list({
    workspaceId: searchParams.workspaceId,
  });

  return databases;
});

export { addCluster, listClusters, deleteCluster, list };
