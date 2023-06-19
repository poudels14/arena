// credit: solid-start
import env from "../../env";
import { useServerContext } from "../context";

/**
 * Links are used to load assets for the server rendered HTML
 * @returns {JSXElement}
 */
export default function Links() {
  const isDev = env.MODE === "development";
  const context = useServerContext();
  // TODO(sagar)
  // !isDev &&
  //   env.ARENA_SSR &&
  //   useAssets(() => getAssetsFromManifest(context!.env.manifest, context!.routerContext!));
  return null;
}
