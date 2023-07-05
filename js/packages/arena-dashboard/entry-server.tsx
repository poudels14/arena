import {
  chainMiddlewares,
  createHandler,
  renderAsync,
} from "@arena/core/server";
import type { PageEvent } from "@arena/core/server";
import { createFileRouter } from "@arena/runtime/filerouter";
import { ServerRoot } from "@arena/core/solid/server";
import { pick } from "lodash-es";
import { router } from "~/api";
import { Context, createContext } from "~/api/context";

const fileRouter = createFileRouter({
  env: {
    SSR: "false",
  },
  resolve: {
    preserveSymlink: true,
  },
});

const handler = chainMiddlewares<{ event: PageEvent; context: Context }>(
  process.env.MODE == "development"
    ? async ({ event }) => fileRouter(event.request)
    : null,
  router({
    prefix: "/api",
  }),
  renderAsync(({ event, context }) => {
    return (
      <ServerRoot
        event={event}
        user={{
          ...pick(context.user, "id", "email", "config"),
          workspaces: context.user.workspaces.map((w) =>
            pick(w, "id", "name", "access")
          ),
        }}
      />
    );
  })
);

const http = createHandler(async (event) => {
  const context = await createContext({
    req: event.request,
    resHeaders: event.request.headers,
  });

  return handler({ event, context });
});

export default {
  fetch: http.fetch,
};
