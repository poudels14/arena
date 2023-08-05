import { inferAsyncReturnType } from "@trpc/server";
import { FetchCreateContextFnOptions } from "@trpc/server/adapters/fetch";
import { Client } from "@arena/runtime/postgres";
import jwt from "@arena/cloud/jwt";
// @ts-expect-error
import cookie from "cookie";
import { createRepo } from "./repos";
import { AclChecker } from "./auth/acl";

let client: Client | null = null;

export async function createContext({
  req,
  resHeaders,
}: FetchCreateContextFnOptions) {
  const repo = await getDbRepo();
  const user = await repo.users.fetchById(parseUserIdFromCookies(req));
  const workspaces = user
    ? await repo.workspaces.listWorkspaces({ userId: user.id })
    : [];

  const userInfo = {
    ...user,
    workspaces,
  } as typeof user & { workspaces: typeof workspaces };
  const acl = new AclChecker(client!, userInfo);

  return {
    req,
    resHeaders,
    user: userInfo,
    repo,
    acl,
    host: process.env.ARENA_HOST || "http://localhost:8000",
  };
}

const getDbRepo = async () => {
  if (!client || !client?.isConnected()) {
    client = new Client({
      credential: process.env.DATABASE_URL,
    });
    await client.connect();
  }
  return createRepo(client);
};

const parseUserIdFromCookies = (req: Request) => {
  const cookies = cookie.parse(req.headers.get("Cookie") || "");
  if (cookies.user) {
    try {
      const { payload } = jwt.verify(
        cookies.user,
        "HS256",
        process.env.JWT_SIGNINIG_SECRET
      );
      return payload.data.id;
    } catch (_) {
      return null;
    }
  }
};

export type Context = inferAsyncReturnType<typeof createContext>;
