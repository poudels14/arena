const pkg = require("./package.json");

/** @type {import('@portal/sdk/app/build').AppConfig} */
module.exports = {
  id: "portal-website",
  version: pkg.version,
  registry: {
    host: "http://localhost:9001/",
    apiKey: process.env.REGISTRY_API_KEY,
  },
};
