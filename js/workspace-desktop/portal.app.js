const path = require("path");
const pkg = require("./package.json");

const version = pkg.version;

/** @type {import('@portal/sdk/app/build').AppConfig} */
module.exports = {
  id: "workspace-desktop",
  version,
  registry: {
    host: "http://localhost:9001/",
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
    ],
  },
  client: {
    input: "./entry-client.tsx",
    minify: true,
    resolve: {
      conditions: ["browser"],
    },
    replace: {
      NODE_ENV: "production",
      MODE: "production",
      PORTAL_STYLE_CSS: `/assets/apps/workspace-desktop/${version}/static/style.css`,
      PORTAL_PUBLISHED_ENTRY_CLIENT: `/assets/apps/workspace-desktop/${version}/static/entry-client.js`,
      PORTAL_ASSETS_BASE: `/assets/apps/workspace-desktop/${version}/static/assets`,
    },
    assets: [path.resolve("./assets")],
  },
  server: {
    input: "./entry-server.tsx",
    minify: true,
    resolve: {
      conditions: ["node"],
    },
    replace: {
      NODE_ENV: "production",
      MODE: "production",
      PORTAL_STYLE_CSS: `/assets/apps/workspace-desktop/${version}/static/style.css`,
      PORTAL_PUBLISHED_ENTRY_CLIENT: `/assets/apps/workspace-desktop/${version}/static/entry-client.js`,
      PORTAL_ASSETS_BASE: `/assets/apps/workspace-desktop/${version}/static/assets`,
    },
  },
};
