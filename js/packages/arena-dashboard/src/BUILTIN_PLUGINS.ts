// TODO(sagar): remove this once the plugin manifests are
// uploaded to db after compilation
import { manifest as classpass } from "@arena/builtins/plugins/classpass";
import { Manifest } from "@arena/sdk/plugins";

const BUILTIN_PLUGINS: Record<string, Manifest> = {
  'export * from "@arena/builtins/plugins/classpass"': classpass,
};

export { BUILTIN_PLUGINS };
