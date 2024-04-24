const pkg = require("./package.json");

/** @type {import('@portal/sdk/app/build').AppConfig} */
module.exports = {
  id: "portal-website",
  version: "2024-04-23-20-15",
  registry: {
    host: "http://localhost:9001/",
    apiKey: process.env.REGISTRY_API_KEY,
  },
};
