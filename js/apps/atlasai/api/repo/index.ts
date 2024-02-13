import { Pool, Client } from "@arena/runtime/postgres";
import { drizzle } from "drizzle-orm/postgres-js";
import { createRepo as createChatThreadsRepo } from "./chatThreads";
import { createRepo as createChatMessagesRepo } from "./chatMessages";
import { createRepo as artifactsRepo } from "./artifacts";
import { createRepo as createTaskExecutionsRepo } from "./tasks";

type Repo = {
  transaction(): Promise<
    Repo & {
      commit(): Promise<void>;
      rollback(): Promise<void>;
    }
  >;
  release(): Promise<void>;
  chatThreads: ReturnType<typeof createChatThreadsRepo>;
  chatMessages: ReturnType<typeof createChatMessagesRepo>;
  artifacts: ReturnType<typeof artifactsRepo>;
  taskExecutions: ReturnType<typeof createTaskExecutionsRepo>;
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
    chatThreads: createChatThreadsRepo(pg),
    chatMessages: createChatMessagesRepo(pg),
    artifacts: artifactsRepo(pg),
    taskExecutions: createTaskExecutionsRepo(pg),
  };
};

export { createRepo };
export type { Repo };
