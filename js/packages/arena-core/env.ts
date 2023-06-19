export default {
  MODE: process.env.MODE || "production",
  SSR: process.env.SSR == "true",
  ARENA_SSR: process.env.ARENA_SSR == "true",
};
