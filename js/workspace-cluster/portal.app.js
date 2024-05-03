const path = require("path");
const pkg = require("./package.json");

const version = pkg.version;

/** @type {import('@portal/sdk/app/build').AppConfig} */
module.exports = {
  id: "workspace-cluster",
  version,
  registry: {
    host: "http://127.0.0.1:9001/",
    apiKey: process.env.REGISTRY_API_KEY,
  },
  resolve: {
    alias: {
      "~/app": path.resolve("./app"),
      "~/api": path.resolve("./api"),
    },
    dedupe: [
      "solid-js",
      "@solidjs/router",
      "@solidjs/meta",
      "@arena/core",
      "@portal/solid-store",
      "@portal/solid-router",
      "@portal/solid-ui",
      "@portal/solid-dnd",
      "@portal/workspace-sdk",
      "@kobalte/core",
      "@portal-apps/assistant",
      "@portal-apps/drive",
      "corvu",
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
