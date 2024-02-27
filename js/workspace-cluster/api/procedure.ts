import { procedure } from "@portal/server-core/router";
import { Pool } from "@arena/runtime/postgres";
import * as jwt from "@arena/cloud/jwt";
import { Client } from "@arena/cloud/s3";
import { Repo } from "./repo";
import { Env } from "./utils/env";

type Context = {
  host: string;
  /**
   * User is always gonna have an id even though the user
   * isn't signed in. When not signed in, the user id will
   * be random and `email` will be null.
   *
   * This allows letting user who isn't signed in perform
   * some actions and then have those actions associated
   * with their account when they signup during the same
   * session
   */
  user: { id: string; email: string | null } | null;
  app: { id: string } | null;
  dbpool: Pool;
  repo: Repo;
  s3Client: Client;
  env: Env;
};

const p = procedure<Context>();

const protectedProcedure = p.use(
  async ({ ctx, next, clearCookie, redirect }) => {
    if (!ctx.user || ctx.user?.id == "public") {
      // Ignore error if user info can't be parsed from the header
      clearCookie("logged-in");
      return redirect("/login");
    }
    return next({
      ctx,
    });
  }
);

const authenticate = p.use(async ({ ctx, next, req, errors }) => {
  const portalUser = req.headers.get("x-portal-user");
  const portalApp = req.headers.get("x-portal-app");
  const user = JSON.parse(portalUser || "null");
  const app = JSON.parse(portalApp || "null");
  return next({
    ctx: {
      ...ctx,
      user,
      app,
    },
  });
});

export { p, authenticate, protectedProcedure };
export type { Context };
