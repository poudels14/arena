import { data } from "./export";

console.log(data);

// test dynamic import
import("./export").then((e) => {
  console.log(e);
});
