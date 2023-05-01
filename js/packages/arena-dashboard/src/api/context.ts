import { inferAsyncReturnType } from "@trpc/server";
import { FetchCreateContextFnOptions } from "@trpc/server/adapters/fetch";
import { Client } from "@arena/postgres";
import { createRepo } from "./repos";

let client: Client | null = null;

export async function createContext({
  req,
  resHeaders,
}: FetchCreateContextFnOptions) {
  const user = {};

  if (!client || !client?.isConnected()) {
    client = new Client({
      connectionString: "postgresql://postgres:password@0.0.0.0:6000/arena",
    });
    await client.connect();
  }
  const repo = createRepo(client);

  return { req, resHeaders, user, repo };
}

export type Context = inferAsyncReturnType<typeof createContext>;
