const version = "0.0.1";

/** @type {import('@portal/sdk/app/build').AppConfig} */
module.exports = {
  id: "workspace-cluster",
  version,
  registry: {
    host: "http://localhost:9009/",
    apiKey: process.env.REGISTRY_API_KEY,
  },
  resolve: {
    alias: {
      "~/app": "./app",
      "~/api": "./api",
    },
    dedupe: [
      "solid-js",
      "@solidjs/router",
      "@solidjs/meta",
      "@arena/core",
      "@portal/solid-store",
      "@portal/solid-router",
      "@portal/solid-ui",
    ],
  },
  client: {
    input: "./entry-client.tsx",
    resolve: {
      conditions: ["browser"],
    },
    replace: {
      NODE_ENV: "production",
      MODE: "production",
      PORTAL_STYLE_CSS: `/registry/apps/workspace-cluster/${version}/static/style.css`,
      PORTAL_PUBLISHED_ENTRY_CLIENT: `/registry/apps/workspace-cluster/${version}/static/entry-client.js`,
    },
  },
  server: {
    input: "./entry-server.tsx",
    resolve: {
      conditions: ["node"],
    },
    replace: {
      NODE_ENV: "production",
      MODE: "production",
      PORTAL_STYLE_CSS: `/registry/apps/workspace-cluster/${version}/static/style.css`,
      PORTAL_PUBLISHED_ENTRY_CLIENT: `/registry/apps/workspace-cluster/${version}/static/entry-client.js`,
    },
  },
};
