"use strict";

((global) => {
  if (!global.Arena) {
    global.Arena = {};
  };

  global.Arena.toInnerResponse = globalThis.__bootstrap.fetch.toInnerResponse;
})(globalThis);
