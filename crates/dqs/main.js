// Note(sagar): need to import these so that deno doesn't complain about
// un-used esm modules
import * as webidl from "ext:deno_webidl/00_webidl.js";
import * as mimesniff from "ext:deno_web/01_mimesniff.js";
import * as global_interfaces from "ext:deno_web/04_global_interfaces.js";
import * as a from "ext:deno_url/01_urlpattern.js";
import * as base64 from "ext:deno_web/05_base64.js";
import * as file from "ext:deno_web/09_file.js";
import * as filereader from "ext:deno_web/10_filereader.js";
import * as location from "ext:deno_web/12_location.js";
import * as message_port from "ext:deno_web/13_message_port.js";
import * as compression from "ext:deno_web/14_compression.js";
import * as headers from "ext:deno_fetch/20_headers.js";
import * as formdata from "ext:deno_fetch/21_formdata.js";
import * as body from "ext:deno_fetch/22_body.js";
import * as http_client from "ext:deno_fetch/22_http_client.js";
import * as request from "ext:deno_fetch/23_request.js";
import * as response from "ext:deno_fetch/23_response.js";
import * as fetch from "ext:deno_fetch/26_fetch.js";

import * as setup from "ext:runtime/setup.js";
import * as http from "ext:runtime/http.js";

globalThis.Arena.core = Deno.core;