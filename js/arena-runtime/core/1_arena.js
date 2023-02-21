"use strict";

// Note(sagar): this is initialized during snapshotting
((global) => {
  if (!global.Arena) {
    global.Arena = {
      // this is populated when build tools are enabled
      BuildTools: {},
    };
  }

  global.Arena.toInnerResponse = globalThis.__bootstrap.fetch.toInnerResponse;
})(globalThis);
