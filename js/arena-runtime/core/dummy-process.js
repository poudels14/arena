"use strict";

((globalThis) => {
  const p = {
    env: {
      TERM: "xterm-256color",
    },
  };

  globalThis.process = p;
})(globalThis);
