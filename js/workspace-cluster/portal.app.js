/** @type {import('@portal/sdk/app/build').BuildConfig} */
module.exports = {
  resolve: {
    alias: {
      "~/api": "./api",
    },
    dedupe: ["@arena/core"],
  },
  server: {
    input: "./server.ts",
    resolve: {
      conditions: ["node"],
    },
    replace: {
      NODE_ENV: "production",
      MODE: "production",
    },
  },
};
