// Note(sagar): need to import these so that deno doesn't complain about
// un-used esm modules
import * as global_interfaces from "ext:deno_web/04_global_interfaces.js";
import * as a from "ext:deno_url/01_urlpattern.js";
import * as base64 from "ext:deno_web/05_base64.js";
import * as filereader from "ext:deno_web/10_filereader.js";
import * as message_port from "ext:deno_web/13_message_port.js";
import * as compression from "ext:deno_web/14_compression.js";
import * as _ from "ext:deno_fetch/27_eventsource.js";

import * as setup from "ext:runtime/0_setup.js";
import * as arena from "ext:runtime/1_arena.js";
import * as process from "ext:runtime/dummy-process.js";
import * as http from "ext:runtime/http.js";

globalThis.Arena.core = Deno.core;
