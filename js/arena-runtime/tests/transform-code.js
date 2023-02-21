const { BuildTools } = Arena;

console.log("************* Transforming inline code ******************");

const inlineCode = BuildTools.transformSync(
  `const x : string = "test string";`
);
console.log(inlineCode);

console.log("************* Transforming code from file ***************");

const code = await BuildTools.transformFileAsync("./typescript/type-export.ts");
console.log(code);
