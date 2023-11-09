import { uniq } from "lodash-es";
import fs from "node:fs";

const file = fs.readFileSync("/home/sagar/Downloads/words.txt", "utf-8");

const words = file.split("\n");

let filterRegex = /^[a-zA-Z]{6}$/;

// console.log(filterRegex.exec("atest"));
// console.log(uniq("asdasq"));
const filteredWords = words.filter((w: string) => {
  if (
    filterRegex.exec(w) &&
    // w.startsWith("sp") &&
    // w.startsWith("ar") &&
    w.startsWith("re") &&
    !["d", "e", "w", "x", "z"].includes(w.substring(0, 1))
  ) {
    // w.length == 4 //|| w.length == 5 || w.length == 6

    return uniq(w.toLowerCase()).length > 3;
  }
  return false;
});

// fs.writeFileSync("w.txt", filteredWords.join("\n"));

// console.log(JSON.stringify(filteredWords));

console.log(filteredWords.join("\n"));
