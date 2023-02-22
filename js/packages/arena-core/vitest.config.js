import { defineConfig } from "vitest/config";

export default defineConfig({
  resolve: {
    conditions: ["browser", "development"],
  },
  test: {
    includeSource: ["src/**/*.{js,ts}"],
    benchmark: {
      includeSource: ["src/**/*.{js,ts}"],
    },
    watch: false,
    reporters: ["basic", "html"],
    outputFile: "./build/tests/report.html",
  },
});
