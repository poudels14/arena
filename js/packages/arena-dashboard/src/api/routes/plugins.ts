import { createRouter, procedure } from "@arena/runtime/server";
import { Context } from "../context";

const PLUGINS = [];

const p = procedure<Context>();
const pluginsRouter = createRouter<any>({
  prefix: "/plugins",
  routes: {
    "/": p.query(async () => {
      return PLUGINS;
    }),
    "/*": p.query(async ({ params, errors }) => {
      const pluginId = params["*"];
      const plugin = PLUGINS.find((p) => p.id == pluginId);
      if (!plugin) {
        errors.notFound("Plugin not found");
      }
      return plugin;
    }),
  },
});

export { pluginsRouter };
