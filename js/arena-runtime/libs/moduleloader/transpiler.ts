import { Transpiler as ArenaTranspiler } from "@arena/runtime/transpiler";
import { babel, presets, plugins } from "@arena/runtime/babel";
import { isMatch } from "matcher";

class Transpiler {
  TRANSPILED_CODE_BY_FILENAME: Map<string, string>;
  TRANSPILED_FILED_LAST_MODIFIED_TIME: Map<string, string>;
  resolverConfig: Arena.ResolverConfig;
  transpiler: ArenaTranspiler;

  constructor(resolverConfig: Arena.ResolverConfig, env: any) {
    this.TRANSPILED_CODE_BY_FILENAME = new Map();
    this.TRANSPILED_FILED_LAST_MODIFIED_TIME = new Map();
    this.resolverConfig = resolverConfig;
    this.transpiler = new ArenaTranspiler({
      resolveImport: true,
      resolver: resolverConfig,
      replace: Object.fromEntries(
        Object.entries(env).flatMap(([k, v]) => {
          return [
            [`Arena.env.${k}`, JSON.stringify(v)],
            [`process.env.${k}`, JSON.stringify(v)],
          ];
        })
      ),
    });
  }

  async transpiledJavascript(filePath: string, ext: string): Promise<string> {
    if (this.TRANSPILED_CODE_BY_FILENAME.has(filePath)) {
      // only return cached transpiled code if the file hasn't been modified
      // after it was transpiled
      const { mtimeMs } = Arena.fs.lstatSync(filePath);
      if (mtimeMs == this.TRANSPILED_FILED_LAST_MODIFIED_TIME.get(filePath)) {
        return this.TRANSPILED_CODE_BY_FILENAME.get(filePath)!;
      }
    }

    const { code } = await this.transpiler.transpileFileAsync(filePath);
    let transpiledCode = code;

    // Note(sagar): further transpile JSX using babel plugins
    if ([".tsx", ".jsx"].includes(ext)) {
      const { code: solidjsCode } = babel.transform(code, {
        presets: [[presets.solidjs, { generate: "dom", hydratable: true }]],
        plugins: [
          [plugins.transformCommonJs, { exportsOnly: true }],
          [plugins.importResolver, this.resolverConfig],
        ],
      });
      transpiledCode = solidjsCode;
    }

    if (isMatch(filePath, "*/node_modules/*")) {
      this.TRANSPILED_CODE_BY_FILENAME.set(filePath, transpiledCode);
      const { mtimeMs } = Arena.fs.lstatSync(filePath);
      this.TRANSPILED_FILED_LAST_MODIFIED_TIME.set(filePath, mtimeMs);
    }
    return transpiledCode;
  }

  async transformedCss(filePath: string, ext: string) {
    const css = await Arena.fs.readToString("./" + filePath);
    const { code } = await this.transpiler.transpileSync(
      `import styleInject from "style-inject";
      const css = \`${css}\`;
      styleInject(css);
      export default css;
      `,
      filePath
    );
    // TODO(sagar): transform using postcss
    return code;
  }
}

export { Transpiler };
