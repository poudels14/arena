import { procedure } from "@portal/server-core/router";
import { Pool } from "@arena/runtime/postgres";
import * as jwt from "@arena/cloud/jwt";
import { Repo } from "./repo";
import { Env } from "./env";

type Context = {
  host: string;
  /**
   * User is always gonna have an id even though the user
   * isn't signed in. When not signed in, the user id will
   * be random and `hasAccount` will be false.
   *
   * This allows letting user who isn't signed in perform
   * some actions and then have those actions associated
   * with their account when they signup during the same
   * session
   */
  user: { id: string; email?: string; hasAccount: boolean } | null;
  dbpool: Pool;
  repo: Repo;
  env: Env;
};

const p = procedure<Context>();

const protectedProcedure = p.use(async ({ ctx, next, errors }) => {
  if (!ctx.user?.hasAccount) {
    return errors.forbidden();
  }
  return next({
    ctx,
  });
});

const authenticate = p.use(async ({ ctx, next, cookies, errors }) => {
  try {
    const { payload } = jwt.verify<{ user: { id: string; email: string } }>(
      cookies.user,
      "HS256",
      ctx.env.JWT_SIGNINIG_SECRET
    );

    const user = await ctx.repo.users.fetchById(payload.user.id);
    if (!user) {
      return errors.forbidden();
    }

    return await next({
      ctx: {
        ...ctx,
        user: {
          id: payload.user.id,
          email: user.email,
          hasAccount: true,
        },
      },
    });
  } catch (e) {
    return errors.forbidden();
  }
});

export { p, authenticate, protectedProcedure };
export type { Context };
