import { Transpiler } from "@arena/runtime/transpiler";

const transpiler = new Transpiler();

const { code: code1 } = transpiler.transpileSync(
  `const x = 11;
   const y = 12;
   exports.x = x;
   exports.y = y;`
);

console.log(code1);

const { code: code2 } = transpiler.transpileSync(
  `const x = 11;
   const y = 12;
   module.exports = { x, y };`
);

console.log(code2);
