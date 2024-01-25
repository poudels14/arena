/** @type {import('@portal/sdk/app/build').AppConfig} */
module.exports = {
  id: "workspace-cluster",
  version: "0.0.2",
  registry: {
    host: "http://localhost:9009/",
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
      PORTAL_STYLE_CSS: `/registry/apps/workspace-cluster/${this.version}/static/style.css`,
      PORTAL_PUBLISHED_ENTRY_CLIENT: `/registry/apps/workspace-cluster/${this.version}/static/entry-client.js`,
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
      PORTAL_STYLE_CSS: `/registry/apps/workspace-cluster/${this.version}/static/style.css`,
      PORTAL_PUBLISHED_ENTRY_CLIENT: `/registry/apps/workspace-cluster/${this.version}/static/entry-client.js`,
    },
  },
};
