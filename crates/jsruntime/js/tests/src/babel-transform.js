const { babel, babelPresets } = Arena;

const { code } = babel.transform(`
  console.log(<div>SOLIDJS element</div>);
`, {
  presets: [
    [babelPresets.solid]
  ]
});

console.log(code)