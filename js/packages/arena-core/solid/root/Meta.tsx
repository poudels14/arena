// credit: solid-start
import { renderTags } from "@solidjs/meta";
import { ssr, useAssets } from "solid-js/web";
import { useServerContext } from "../context";

export default function Meta() {
  const { event } = useServerContext();
  // @ts-expect-error The ssr() types do not match the Assets child types
  useAssets(() => ssr(renderTags(event.tags)));
  return null;
}
