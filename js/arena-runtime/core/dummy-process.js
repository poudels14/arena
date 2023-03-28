"use strict";

// Note(sagar): this is just a dummy process that can be used
// when node modules aren't enabled
((globalThis) => {
  globalThis.process = globalThis.process ?? {
    env: {
      TERM: "xterm-256color",
    },
  };
})(globalThis);
