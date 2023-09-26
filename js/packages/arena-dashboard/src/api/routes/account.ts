import { createRouter, procedure } from "@arena/runtime/server";
import * as jwt from "@arena/cloud/jwt";
import ms from "ms";
import { pick } from "lodash-es";
import ky from "ky";
import { renderToString } from "@arena/email";
import { Context } from "../context";
import { Login } from "../emails/Login";

const p = procedure<Context>();
const accountRouter = createRouter<any>({
  prefix: "/account",
  routes: {
    /**
     * Send magic link to the email
     */
    "/login": p.mutate(async ({ req, ctx, errors }) => {
      const { email } = (await req.json().catch((e) => {})) || {};
      if (!email) {
        return errors.badRequest('HTTP Post body must contain "email" field');
      }
      // TODO(sagar): use CSRF, rate limiting, etc to prevent DDOS

      const user = await ctx.repo.users.fetchByEmail(email);
      if (!user) {
        return errors.badRequest();
      }

      const signInToken = jwt.sign({
        header: { alg: "HS256" },
        payload: {
          user: {
            id: user.id,
          },
          exp: (new Date().getTime() + ms("2 weeks")) / 1000,
        },
        secret: process.env.JWT_SIGNINIG_SECRET,
      });

      try {
        const json = await ky
          .post("https://api.resend.com/emails", {
            headers: {
              Authorization: `Bearer ${process.env.RESEND_API_KEY}`,
            },
            json: {
              from: "Sign in to Arena <signin@emails.tryarena.io>",
              to: "poudels14@gmail.com",
              subject: "Sign in to Arena",
              html: renderToString(
                Login({
                  magicLink: `${ctx.host}/api/account/login/email?magicToken=${signInToken}`,
                })
              ),
            },
          })
          .json();
        // TODO(sagar): show message saying login-link has been sent
        return "Ok";
      } catch (e) {
        console.error(e);
        throw e;
      }
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

          const { id: userId } = payload.user || {};

          let user;
          if (!userId || !(user = await ctx.repo.users.fetchById(userId))) {
            return errors.badRequest();
          }

          const workspaces = await ctx.repo.workspaces.listWorkspaces({
            userId: user.id,
          });

          // if there's no workspace for the user, create one
          if (workspaces.length == 0) {
            await ctx.repo.workspaces.createWorkspaceForUser(user.id);
          }

          const signInToken = jwt.sign({
            header: { alg: "HS256" },
            payload: {
              user: {
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
