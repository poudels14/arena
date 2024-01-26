import ms from "ms";
import ky from "ky";
import { pick } from "lodash-es";
import { renderToString } from "@portal/email";
import * as jwt from "@arena/cloud/jwt";
import z from "zod";
import { Login } from "./emails/Login";
import { p } from "./procedure";
import { uniqueId } from "@portal/sdk/utils/uniqueId";
import { addApp } from "./utils/app";

const signup = p
  .input(
    z.object({
      email: z.string(),
    })
  )
  .mutate(async ({ ctx, errors }) => {
    // TODO
    return errors.notFound();
  });

/**
 * Send magic link to the email
 */
const sendMagicLink = p
  .input(
    z.object({
      email: z.string(),
    })
  )
  .mutate(async ({ ctx, body, errors }) => {
    // TODO(sagar): use CSRF, rate limiting, etc to prevent DDOS
    const user = await ctx.repo.users.fetchByEmail(body.email);
    if (!user) {
      return errors.forbidden();
    }

    const signInToken = jwt.sign({
      header: { alg: "HS256" },
      payload: {
        user: {
          id: user.id,
        },
        // Create a short lived token since this is sent to an email
        exp: (new Date().getTime() + ms("10 minutes")) / 1000,
      },
      secret: ctx.env.JWT_SIGNING_SECRET,
    });

    if (ctx.env.MODE == "development") {
      console.log(
        `${ctx.host}/api/account/login/magic?magicToken=${signInToken}`
      );
    }
    try {
      await ky
        .post("https://api.resend.com/emails", {
          headers: {
            Authorization: `Bearer ${ctx.env.RESEND_API_KEY}`,
          },
          json: {
            from: `Sign in to Portal <${ctx.env.LOGIN_EMAIL_SENDER}>`,
            to: "poudels14@gmail.com",
            subject: "Sign in to Arena",
            html: renderToString(
              Login({
                host: ctx.host,
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
  });

const magicLinkLogin = p.query(
  async ({ ctx, searchParams, setCookie, errors, redirect }) => {
    const { magicToken } = searchParams;
    if (!magicToken) {
      return errors.badRequest();
    }

    try {
      const { payload } = jwt.verify(
        magicToken,
        "HS256",
        ctx.env.JWT_SIGNING_SECRET
      );

      const { id: userId } = payload.user || {};

      let user;
      if (!userId || !(user = await ctx.repo.users.fetchById(userId))) {
        console.log("Magic token login error: payload =", payload);
        return errors.badRequest();
      }

      const workspaces = await ctx.repo.workspaces.listWorkspaces({
        userId: user.id,
      });

      // if there's no workspace for the user, create one
      if (workspaces.length == 0) {
        const workspace = await ctx.repo.workspaces.createWorkspace({
          id: uniqueId(14),
          ownerId: user.id,
        });

        const repo = await ctx.repo.transaction();
        const atlasAi = await ctx.repo.appTemplates.fetchById("atlasai");
        if (atlasAi) {
          await addApp(
            repo,
            { id: userId },
            {
              id: uniqueId(14),
              workspaceId: workspace.id,
              name: "Atlas AI",
              description: "An AI Assistant",
              template: {
                id: atlasAi.id,
                version: atlasAi.defaultVersion || "0.0.1",
              },
            }
          );
        }
        await repo.commit();
        await repo.release();
      }

      const signInToken = jwt.sign({
        header: { alg: "HS256" },
        payload: {
          user: {
            id: user.id,
            email: user.email,
            config: pick(user.config, "waitlisted"),
          },
          exp: (new Date().getTime() + ms("4 weeks")) / 1000,
        },
        secret: ctx.env.JWT_SIGNING_SECRET,
      });
      setCookie("user", signInToken, {
        path: "/",
      });
      return redirect("/");
    } catch (e) {
      console.log(e);
      return errors.badRequest();
    }
  }
);

export { signup, sendMagicLink, magicLinkLogin };
