// Note(sagar): node modules should be enabled for this
import assert from "assert";
import { resolve } from "path";
import { lstat, readFile } from "fs/promises";

const resolved = resolve("./simple.js");

const test = async () => {
  console.log("Resolved =", resolved);
  console.log(typeof resolved);
  assert.equal(typeof resolved, "string");

  const stat = await lstat(resolved);
  assert.equal(stat.isSymlink, false);
  console.log("lstat =", stat);

  const fileContent = await readFile(resolved);
  assert.equal(Object.getPrototypeOf(fileContent), ArrayBuffer.prototype);

  const stringContent = await readFile(resolved, "utf-8");
  assert.equal(Object.getPrototypeOf(stringContent), String.prototype);

  console.log("File content {utf-8} =", stringContent);
};

test();
