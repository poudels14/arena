import { pick } from "lodash-es";
import type { Manifest } from "@arena/sdk/plugins";
import { p } from "../procedure";
import { DocumentEmbeddingsGenerator } from "../EmbeddingsGenerator";

const embeddingsGenerator = new DocumentEmbeddingsGenerator();

const listPlugins = p.query(async ({ ctx }) => {
  const { default: sql } = ctx.dbs;
  const { rows: plugins } = await sql.query<any>(`SELECT * FROM plugins`);
  return plugins.map((plugin) => {
    return {
      ...pick(plugin, "id", "name", "version"),
      installedAt: new Date(plugin.installedAt).toISOString(),
    };
  });
});

const searchPlugins = p.query(async ({ ctx, searchParams, errors }) => {
  const db = ctx.dbs.vectordb;

  if (!searchParams.query) {
    return errors.badRequest("Invalid search query");
  }

  const generator = new DocumentEmbeddingsGenerator();
  const embeddings = await generator.getTextEmbeddings([searchParams.query]);
  const results = await db.searchCollection("plugins", embeddings[0], 5, {
    includeChunkContent: true,
    contentEncoding: "utf-8",
    minScore: 0.75,
  });

  return results.map((r) => {
    const slashIndex = r.documentId.lastIndexOf("/");
    const pluginId = r.documentId.substring(0, slashIndex);
    const functionName = r.documentId.substring(slashIndex);
    return {
      score: r.score,
      pluginId,
      function: {
        slug: functionName,
        ...JSON.parse(r.content),
      },
    };
  });
});

const installPlugin = p.mutate(async ({ req, ctx, errors }) => {
  const { default: mainDb, vectordb } = ctx.dbs;
  const pluginsToInstall = (await req.json()) as { id: string }[];

  const { rows: existingPlugins } = await mainDb.query<{ id: string }>(
    `SELECT * FROM plugins WHERE id = ?`,
    pluginsToInstall.map((p) => p.id)
  );

  const newPlugins = pluginsToInstall.filter(
    (p) => !existingPlugins.find((ep) => ep.id !== p.id)
  );

  if (newPlugins.length > 1) {
    return errors.badRequest("Only one plugin can be installed at a time");
  }

  const pluginId = newPlugins[0].id;

  const manifestReq = await fetch(
    `${process.env.ARENA_HOST}/api/plugins/${pluginId}`
  );

  if (manifestReq.status !== 200) {
    return errors.notFound(`Couldn't find plugin: ${pluginId}`);
  }

  const manifest: Manifest = await manifestReq.json();
  const { llm, workflows } = manifest;
  await Promise.all(
    workflows!.map(async (query) => {
      const queryId = `${pluginId}/${query.slug}`;
      const content = JSON.stringify(query);

      await vectordb.deleteDocument("plugins", queryId).catch(() => {});
      await vectordb.addDocument("plugins", queryId, {
        content,
        metadata: {
          type: "query",
        },
      });

      const descriptions = [query.description, ...(query.queryPrompts || [])];
      const embeddings = await embeddingsGenerator.getChunkEmbeddings(
        // TODO(sagar): we dont want duplicate quries in topk results,
        // so figure out a way to achive that
        descriptions.map((description) => {
          const queryDescription = llm.description + "\n" + description;
          return {
            content: queryDescription,
            position: {
              start: 0,
              end: content.length,
            },
          };
        })
      );

      await vectordb.setDocumentEmbeddings("plugins", queryId, embeddings);
    })
  );

  return {
    success: true,
    plugin: pick(p, "id", "name", "version", "installedAt"),
  };
});

const uninstallPlugin = p.delete(async ({ ctx, params }) => {
  // TODO
  // const { default: mainDb, vectordb } = ctx.dbs;
  // const queryId = null;
  // await mainDb.transaction(async () => {
  //   await vectordb.deleteDocument("plugins", queryId);
  //   await mainDb.query<any>(`DELETE FROM plugins WHERE id = ?`, [
  //     params.pluginId,
  //   ]);
  // });
  // return { success: true };
});

export { listPlugins, searchPlugins, installPlugin, uninstallPlugin };
