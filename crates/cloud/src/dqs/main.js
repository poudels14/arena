// Note(sagar): need to import these so that deno doesn't complain about
// un-used esm modules
import * as global_interfaces from "ext:deno_web/04_global_interfaces.js";
import * as a from "ext:deno_url/01_urlpattern.js";
import * as base64 from "ext:deno_web/05_base64.js";
import * as filereader from "ext:deno_web/10_filereader.js";
import * as message_port from "ext:deno_web/13_message_port.js";
import * as compression from "ext:deno_web/14_compression.js";
import * as image_data from "ext:deno_web/16_image_data.js";
import * as eventsource from "ext:deno_fetch/27_eventsource.js";

import * as setup from "ext:runtime/setup.js";
import * as http from "ext:runtime/http.js";

globalThis.Arena.core = Deno.core;
