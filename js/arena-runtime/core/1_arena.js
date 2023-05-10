import { toInnerResponse } from "ext:deno_fetch/23_response.js";

// Note(sagar): this is initialized during snapshotting
((global) => {
  global.Arena = global.Arena ?? {};

  global.Arena.toInnerResponse = toInnerResponse;
})(globalThis);
