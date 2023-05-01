// Note(sagar): these only work if node modules are enabled in the runtime
import assert from "node:assert";
import { dirname, resolve, join } from "node:path";

assert.equal(dirname("./parent/file.js"), "./parent");
assert.equal(resolve("./simple.js"), join(process.cwd(), "./simple.js"));
