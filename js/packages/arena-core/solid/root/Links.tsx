// credit: solid-start
import { useContext } from "solid-js";
import { ServerContext } from "../context";

/**
 * Links are used to load assets for the server rendered HTML
 * @returns {JSXElement}
 */
export default function Links() {
  const isDev = process.env.MODE === "development";
  const context = useContext(ServerContext);
  // TODO(sagar)
  // !isDev &&
  //   process.env.ARENA_SSR &&
  //   useAssets(() => getAssetsFromManifest(context!.env.manifest, context!.routerContext!));
  return null;
}
