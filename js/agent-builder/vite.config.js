import { vitePlugin as remix } from "@remix-run/dev";
import { defineConfig } from "vite";

export default defineConfig({
  build: {
    rollupOptions: {
      input: "./server/index.ts",
    },
  },
  plugins: [remix()],
});
