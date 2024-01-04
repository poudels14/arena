import { chainMiddlewares, createHandler } from "@portal/server-core";
import type { PageEvent } from "@portal/server-core";
import { createDefaultFileRouter } from "@portal/server-dev/solidjs";
import { ServerRoot, renderToStringAsync } from "@portal/solidjs/server";
import Root from "~/app/root";
import { router } from "~/api/index";

const fileRouter = await createDefaultFileRouter({
  baseDir: process.cwd(),
  env: {
    SSR: "false",
    PORTAL_SSR: "false",
    PORTAL_ENTRY_CLIENT: "./entry-client.tsx",
  },
  babel: {},
  resolverConfig: {
    preserveSymlink: true,
    alias: {
      "~": "./app",
    },
    conditions: ["solid", "browser"],
    dedupe: [
      "solid-js",
      "@solidjs/router",
      "@solidjs/meta",
      "@portal/solid-store",
    ],
  },
  transpilerConfig: {
    resolveImports: true,
  },
});

const handler = chainMiddlewares<{ event: PageEvent }>(
  async ({ event }) => {
    const res = await fileRouter.route(event.request);
    if (res && res.status != 404) {
      return res;
    }
  },
  async ({ event }) => {
    try {
      return await router.route(event.request, {
        env: event.env,
        context: {
          user: {
            id: null,
          },
        },
      });
    } catch (e) {
      console.error(e);
    }
  },
  renderToStringAsync(({ event }) => {
    return <ServerRoot event={event} Root={Root} />;
  })
);

const http = createHandler(async (event) => await handler({ event }));
export default {
  fetch: http.fetch,
};
