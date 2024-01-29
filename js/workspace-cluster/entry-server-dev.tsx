import { createDefaultFileRouter } from "@portal/server-dev/solidjs";
import mainEntry from "./entry-server";

const fileRouter = await createDefaultFileRouter({
  baseDir: process.cwd(),
  env: {
    NODE_ENV: "development",
    SSR: "false",
    PORTAL_SSR: "false",
    PORTAL_ENTRY_CLIENT: "./entry-client.tsx",
    PORTAL_STYLE_CSS: "./static/style.css",
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
      "@arena/core",
      "@portal/solid-store",
      "@portal/solid-router",
      "@portal/solid-query",
      "@portal/solidjs",
    ],
  },
  transpilerConfig: {
    resolveImports: true,
  },
});

export default {
  async fetch(request: Request) {
    const url = new URL(request.url);
    if (!url.pathname.startsWith("/api")) {
      const res = await fileRouter.route(request);
      if (res && res.status != 404) {
        return res;
      }
    }
    return await mainEntry.fetch(request);
  },
};
