import { toInnerResponse } from "ext:deno_fetch/23_response.js";

// Note(sagar): this is initialized during snapshotting
((global) => {
  if (!global.Arena) {
    global.Arena = {
      // this is populated when build tools are enabled
      BuildTools: {},
    };
  }

  global.Arena.toInnerResponse = toInnerResponse;
})(globalThis);
