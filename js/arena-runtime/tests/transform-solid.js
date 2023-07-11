import { Transpiler } from "@arena/runtime/transpiler";

const transpiler = new Transpiler();
const { code } = transpiler.transpileSync(
  `const Component = () => <div>hello</div>`
);
console.log(code);
