// Note(sagar): need to import these so that deno doesn't complain about
// un-used esm modules
import * as a from "ext:deno_url/01_urlpattern.js";
import * as base64 from "ext:deno_web/05_base64.js";
import * as filereader from "ext:deno_web/10_filereader.js";
import * as message_port from "ext:deno_web/13_message_port.js";
import * as compression from "ext:deno_web/14_compression.js";
import * as perf_15 from "ext:deno_web/15_performance.js";
import * as image_16 from "ext:deno_web/16_image_data.js";

import * as http from "ext:runtime/http.js";
import * as setup1 from "ext:runtime/0_setup.js";
import * as arena from "ext:runtime/1_arena.js";
import * as process from "ext:runtime/dummy-process.js";

globalThis.global = new Proxy(globalThis, {
  get(a, p) {
    return globalThis[p];
  },
});

globalThis.Arena.core = Deno.core;
