/** @type {import('@portal/sdk/app/build').AppConfig} */
module.exports = {
  id: "portal-website",
  version: "0.0.1",
  registry: {
    host: "http://localhost:9009/",
    apiKey: process.env.REGISTRY_API_KEY,
  },
};
