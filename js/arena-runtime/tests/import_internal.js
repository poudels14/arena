// Node(sagar): importing extension module should be blocked
import { TextEncoder } from "ext:deno_web/08_text_encoding.js";

console.log(TextEncoder);
