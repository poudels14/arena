import path from "path";
import { build } from "@arena/workspace-server";

await Arena.fs
  .readAsJson(path.join(process.cwd(), "./workspace.config.toml"))
  .then((config) => {
    console.log("Config loaded from ./workspace.config.toml");
    return JSON.parse(config);
  })
  .catch((_) => {
    console.log("Error loading workspace.config.toml. Using default configs");
    return {};
  })
  .then(async (config) => {
    await build({
      outDir: "build/",
      client: {
        entry: config.client.entry,
        config: {
          env: config.client.env,
          javascript: config.client.javascript,
        },
        minify: true,
      },
      server: {
        entry: config.server.entry,
        config: {
          env: config.server.env,
          javascript: config.server.javascript,
        },
      },
    });
  });
