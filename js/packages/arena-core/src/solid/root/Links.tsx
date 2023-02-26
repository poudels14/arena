// credit: solid-start
import { useContext } from "solid-js";
import { ServerContext } from "../server";

/**
 * Links are used to load assets for the server rendered HTML
 * @returns {JSXElement}
 */
export default function Links() {
  const isDev = Arena.env.MODE === "development";
  const context = useContext(ServerContext);
  // TODO(sagar)
  // !isDev &&
  //   Arena.env.ARENA_SSR &&
  //   useAssets(() => getAssetsFromManifest(context!.env.manifest, context!.routerContext!));
  return null;
}
