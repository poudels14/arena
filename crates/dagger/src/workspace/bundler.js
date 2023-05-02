import path from "path";
import { build } from "@arena/workspace-server/builder";

const bundle = async (options) => {
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
      const { client, server } = config;
      await build({
        outDir: "build/",
        client: {
          entry: client.entry,
          config: {
            env: client.env,
            javascript: client.javascript,
          },
          minify: options.minify,
        },
        server: {
          entry: server.entry,
          config: {
            env: server.env,
            javascript: server.javascript,
          },
        },
      });
    });
};

export { bundle };
