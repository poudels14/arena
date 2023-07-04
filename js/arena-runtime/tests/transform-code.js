import { Transpiler } from "@arena/runtime/transpiler";

console.log("************* Transforming inline code ******************");

const transpiler = new Transpiler({
  resolveImport: true,
});
const inlineCode = transpiler.transpileSync(
  `const x : string = "test string";`
);
console.log(inlineCode);

console.log(
  transpiler.transpileSync('const AIChat = lazy(() => import("./simple.js"))')
);

console.log("************* Transforming code from file ***************");

const code = await transpiler.transpileFileAsync("./typescript/type-export.ts");
console.log(code);
