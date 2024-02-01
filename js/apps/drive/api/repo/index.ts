import { Pool, Client } from "@arena/runtime/postgres";
import { drizzle } from "drizzle-orm/postgres-js";
import { createRepo as createFilesRepo } from "./files";
import { createRepo as createEmbeddingsRepo } from "./embeddings";

type Repo = {
  transaction(): Promise<
    Repo & {
      commit(): Promise<void>;
      rollback(): Promise<void>;
    }
  >;
  release(): Promise<void>;
  files: ReturnType<typeof createFilesRepo>;
  embeddings: ReturnType<typeof createEmbeddingsRepo>;
};

const createRepo = async (options: { pool?: Pool; client?: Client }) => {
  const client = options.client ?? (await options.pool!.connect());
  let pg = drizzle(client);

  return {
    async transaction() {
      if (!options.pool) {
        throw new Error("Database pool must be passed when creating a repo");
      }
      const client = await options.pool!.connect();
      await client.query("BEGIN");
      const repo = await createRepo({ client });
      return Object.assign(repo, {
        async transaction() {
          throw new Error("Nested transaction not supported");
        },
        async commit() {
          await client.query("COMMIT");
        },
        async rollback() {
          await client.query("ROLLBACK");
        },
      });
    },
    async release() {
      // rollback just in case the client was release
      // in the middle of the transaction
      await client.query("ROLLBACK");
      // @ts-expect-error
      client?.release && client?.release();
    },
    files: createFilesRepo(pg),
    embeddings: createEmbeddingsRepo(pg),
  };
};

export { createRepo };
export type { Repo };
