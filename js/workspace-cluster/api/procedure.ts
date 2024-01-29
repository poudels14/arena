import { procedure } from "@portal/server-core/router";
import { Pool } from "@arena/runtime/postgres";
import * as jwt from "@arena/cloud/jwt";
import { Client } from "@arena/cloud/s3";
import { Repo } from "./repo";
import { Env } from "./env";

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
  dbpool: Pool;
  repo: Repo;
  s3Client: Client;
  env: Env;
};

const p = procedure<Context>();

const protectedProcedure = p.use(async ({ ctx, next, errors }) => {
  if (!ctx.user?.email) {
    return errors.forbidden();
  }
  return next({
    ctx,
  });
});

const parseUserFromHeaders = p.use(
  async ({ req, ctx, next, cookies, clearCookie, redirect }) => {
    if (process.env.DISABLE_AUTH == "true") {
      return await next({
        ctx: {
          ...ctx,
          user: {
            id: "test-user-dev",
            email: "test-user@test.com",
          },
        },
      });
    }

    const token = cookies.user || req.headers.get("x-portal-authentication");
    let user = null;
    try {
      const { payload } = jwt.verify<{ user: { id: string; email: string } }>(
        token || "",
        "HS256",
        ctx.env.JWT_SIGNING_SECRET
      );
      user = await ctx.repo.users.fetchById(payload.user.id);
    } catch (e) {
      // Ignore error if user info can't be parsed from the header
      clearCookie("logged-in");
      return redirect("/login");
    }

    return await next({
      ctx: {
        ...ctx,
        user,
      },
    });
  }
);

const authenticate = parseUserFromHeaders.use(async ({ ctx, next, errors }) => {
  if (!ctx.user?.email) {
    return errors.forbidden();
  }
  return next({
    ctx,
  });
});

export { p, parseUserFromHeaders, authenticate, protectedProcedure };
export type { Context };
