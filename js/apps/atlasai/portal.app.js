const path = require("path");

/** @type {import('@portal/sdk/app/build').AppConfig} */
module.exports = {
  id: "atlasai",
  version: "0.0.1",
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
    ],
  },
  server: {
    input: "./entry-server.tsx",
    minify: true,
    resolve: {
      conditions: ["browser"],
    },
    replace: {
      NODE_ENV: "production",
      MODE: "production",
      SSR: "true",
      PORTAL_SSR: "false",
    },
  },
};
