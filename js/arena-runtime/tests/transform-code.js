const { Transpiler } = Arena.BuildTools;

console.log("************* Transforming inline code ******************");

const inlineCode = Transpiler.transformSync(
  `const x : string = "test string";`
);
console.log(inlineCode);

console.log("************* Transforming code from file ***************");

const code = await Transpiler.transformFileAsync("./typescript/type-export.ts");
console.log(code);
