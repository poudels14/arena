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
import { User } from "./repo/users";

const findUser = p
  .input(
    z.object({
      email: z.string(),
    })
  )
  // TODO: add more auth layer to prevent malacious users from crawling this url
  .mutate(async ({ ctx, body, errors }) => {
    const user = await ctx.repo.users.fetchByEmail(body.email);
    if (!user) {
      return errors.notFound();
    }
    return pick(user, "id", "email", "firstName", "lastName");
  });

// TODO: add more auth layer to prevent malacious users from crawling this url
const listUsers = p.query(async ({ ctx, searchParams }) => {
  if (!searchParams.id) {
    return [];
  }
  const ids =
    typeof searchParams.id == "string" ? [searchParams.id] : searchParams.id;
  const users = await ctx.repo.users.fetchByIds(ids);
  return users.map((user) =>
    pick(user, "id", "email", "firstName", "lastName")
  );
});

const signup = p
  .input(
    z.object({
      firstName: z.string(),
      lastName: z.string(),
      email: z.string(),
      message: z.string().optional(),
    })
  )
  .mutate(async ({ ctx, body, errors }) => {
    if (!body.firstName) {
      return errors.badRequest({ error: "First name required" });
    }
    if (!body.lastName) {
      return errors.badRequest({ error: "Last name required" });
    }
    if (!body.email) {
      return errors.badRequest({ error: "Email required" });
    }
    let user = await ctx.repo.users.fetchByEmail(body.email);
    if (user) {
      return errors.badRequest({ error: "Account already exists" });
    }
    user = await ctx.repo.users.insert({
      id: "u-" + uniqueId(16),
      firstName: body.firstName,
      lastName: body.lastName,
      email: body.email,
      config: {
        message: body.message,
      },
    });
    return { id: user.id };
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
  .mutate(async ({ ctx, body, errors, redirect }) => {
    // TODO(sagar): use CSRF, rate limiting, etc to prevent DDOS
    let user = await ctx.repo.users.fetchByEmail(body.email);
    if (!user) {
      return errors.notFound("User not found");
    }
    if (user.config.waitlisted) {
      return redirect("/waitlisted");
    }

    const signInToken = jwt.sign({
      header: { alg: "HS512" },
      payload: {
        user: {
          id: user.id,
        },
        // Create a short lived token since this is sent to an email
        exp: (new Date().getTime() + ms("10 minutes")) / 1000,
      },
      secret: ctx.env.JWT_SIGNING_SECRET,
    });

    const loginLink = new URL(
      `/api/account/login/magic?magicToken=${signInToken}`,
      ctx.host
    );

    if (ctx.env.MODE == "development") {
      console.log(loginLink.toString());
    }
    try {
      await ky
        .post("https://api.resend.com/emails", {
          headers: {
            Authorization: `Bearer ${ctx.env.RESEND_API_KEY}`,
          },
          json: {
            from: `Sign in to Sidecar <${ctx.env.LOGIN_EMAIL_SENDER}>`,
            to: user.email,
            subject: "Sign in to Arena",
            html: renderToString(
              Login({
                host: ctx.host,
                magicLink: loginLink.toString(),
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
      return errors.badRequest("Missing magicToken search param");
    }

    try {
      const { payload } = jwt.verify(
        magicToken,
        "HS512",
        ctx.env.JWT_SIGNING_SECRET
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
        const workspace = await ctx.repo.workspaces.createWorkspace({
          id: uniqueId(19),
          ownerId: user.id,
          config: {
            runtime: {
              netPermissions: {
                // No restrictions by default
                restrictedUrls: [],
              },
            },
          },
        });

        const repo = await ctx.repo.transaction();
        try {
          const atlasAi = await ctx.repo.appTemplates.fetchById("atlasai");
          if (atlasAi) {
            await addApp(
              repo,
              { id: userId },
              {
                id: uniqueId(19),
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
          const portalDrive = await ctx.repo.appTemplates.fetchById(
            "portal-drive"
          );
          if (portalDrive) {
            await addApp(
              repo,
              { id: userId },
              {
                id: uniqueId(19),
                workspaceId: workspace.id,
                name: "Portal Drive",
                description: "Portal Drive",
                template: {
                  id: portalDrive.id,
                  version: portalDrive.defaultVersion || "0.0.1",
                },
              }
            );
          }
          await repo.commit();
        } finally {
          await repo.release();
        }
      }

      const signInToken = createSignInToken(ctx.env.JWT_SIGNING_SECRET, user);
      setCookie("logged-in", "true", {
        path: "/",
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

const createSignInToken = (secret: string, user: User) => {
  return jwt.sign({
    header: { alg: "HS512" },
    payload: {
      user: {
        id: user.id,
        email: user.email,
        config: pick(user.config, "waitlisted"),
      },
      exp: (new Date().getTime() + ms("4 weeks")) / 1000,
    },
    secret,
  });
};

export { findUser, listUsers, signup, sendMagicLink, magicLinkLogin };
