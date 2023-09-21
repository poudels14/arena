import assert from "assert";
import { lstat, readFile } from "builtin://fs/promises";

const run = async () => {
  console.log("CWD =", Arena.fs.cwdSync());
  assert.equal(Arena.fs.existsSync("./simple.js"), true);

  // Note(sagar): this script should be run from "tests" directory
  const { fs } = Arena;
  const file = await fs.readFile("./simple.js", "utf8");
  console.log(file);
};

run();
