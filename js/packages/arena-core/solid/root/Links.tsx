// credit: solid-start
import { useServerContext } from "../context";

/**
 * Links are used to load assets for the server rendered HTML
 * @returns {JSXElement}
 */
export default function Links() {
  const isDev = process.env.MODE === "development";
  const context = useServerContext();
  // TODO(sagar)
  // !isDev &&
  //   process.env.ARENA_SSR &&
  //   useAssets(() => getAssetsFromManifest(context!.env.manifest, context!.routerContext!));
  return null;
}
