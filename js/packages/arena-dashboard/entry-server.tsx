import {
  chainMiddlewares,
  createHandler,
  renderAsync,
} from "@arena/core/server";
import type { PageEvent } from "@arena/core/server";
import { ServerRoot } from "@arena/core/solid/server";
import { pick } from "lodash-es";
import { router } from "~/api";
import { Context, createContext } from "~/api/context";

const handler = chainMiddlewares<{ event: PageEvent; context: Context }>(
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

  if (event.ctx.path == "/ws" && context.user) {
    return new Response(JSON.stringify(context.user), {
      headers: [["Upgrade", "websocket"]],
    });
  }

  return handler({ event, context });
});

export default {
  fetch: http.fetch,
  async websocket(socket: any, data: any) {},
};
