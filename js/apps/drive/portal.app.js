const path = require("path");
const pkg = require("./package.json");

/** @type {import('@portal/sdk/app/build').AppConfig} */
module.exports = {
  id: "portal-drive",
  version: pkg.version,
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
    ],
  },
  server: {
    input: "./entry-server.tsx",
    minify: true,
    resolve: {
      conditions: ["solid", "node"],
    },
    replace: {
      NODE_ENV: "production",
      MODE: "production",
      SSR: "true",
      PORTAL_SSR: "false",
    },
  },
};
