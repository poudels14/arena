import { babel, presets } from "@arena/runtime/babel";

const { code } = babel.transform(
  `
  console.log(<div>SOLIDJS element</div>);
`,
  {
    presets: [[presets.solidjs]],
  }
);

console.log(code);
