import { createRouter, procedure } from "@arena/runtime/server";
import jwt from "@arena/cloud/jwt";
// @ts-expect-error
import ms from "ms";
import { Context } from "../context";
import { pick } from "lodash-es";

const p = procedure<Context>();
const accountRouter = createRouter<any>({
  prefix: "/account",
  routes: {
    /**
     * Send magic link to the email
     */
    "/login": p.mutate(async ({ req, ctx, errors }) => {
      const { email } = (await req.json()) || {};
      if (!email) {
        return errors.badRequest();
      }
      // TODO(sagar): use CSRF, rate limiting, etc to prevent DDOS

      const user = await ctx.repo.users.fetchByEmail(email);
      if (!user) {
        return errors.badRequest();
      }

      const signInToken = jwt.sign({
        header: { alg: "HS256" },
        payload: {
          data: {
            userId: user.id,
          },
          exp: (new Date().getTime() + ms("2 weeks")) / 1000,
        },
        secret: process.env.JWT_SIGNINIG_SECRET,
      });

      return {
        token: signInToken,
      };
    }),
    "/login/email": p.query(
      async ({ req, ctx, searchParams, setCookie, errors, redirect }) => {
        const { magicToken } = searchParams;
        if (!magicToken) {
          return errors.badRequest();
        }

        try {
          const { payload } = jwt.verify(
            magicToken,
            "HS256",
            process.env.JWT_SIGNINIG_SECRET
          );

          const { userId } = payload.data || {};

          let user;
          if (!userId || !(user = await ctx.repo.users.fetchById(userId))) {
            return errors.badRequest();
          }

          const signInToken = jwt.sign({
            header: { alg: "HS256" },
            payload: {
              data: {
                id: user.id,
                email: user.email,
                config: pick(user.config, "waitlisted"),
              },
              exp: (new Date().getTime() + ms("2 weeks")) / 1000,
            },
            secret: process.env.JWT_SIGNINIG_SECRET,
          });
          setCookie("user", signInToken, {
            path: "/",
          });
          return redirect("/");
        } catch (e) {
          return errors.badRequest();
        }
      }
    ),
  },
});

export { accountRouter };
