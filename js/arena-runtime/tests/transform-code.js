const { Transpiler } = Arena.BuildTools;

console.log("************* Transforming inline code ******************");

console.log(Arena);

console.log(Transpiler);

const transpiler = new Transpiler();
const inlineCode = transpiler.transpileSync(
  `const x : string = "test string";`
);
console.log(inlineCode);

console.log("************* Transforming code from file ***************");

const code = await transpiler.transpileFileAsync("./typescript/type-export.ts");
console.log(code);
