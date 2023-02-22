declare namespace Arena {
  type Core = {
    ops: Record<string, any>;
  };

  type Env = Record<string, any>;

  type TransformOptions = {
    source_map?: "inline";
  };

  type TransformResult = {
    code: string;
  };

  type BuildTools = {
    babel: any;
    babelPlugins: any;
    babelPresets: {
      solid: any;
    };

    transformFileAsync: (
      filename: string,
      options: TransformOptions
    ) => Promise<TransformResult>;
    transformSync: (code: string, options: TransformOptions) => TransformResult;
  };

  let core: Core;
  let env: Env;
  let BuildTools: BuildTools;
  let wasi: any;

  let toInnerResponse: (response: Response) => any;
}

interface ImportMeta {
  url: string;

  /**
   * Return the resolved absolute path of the given path/module
   */
  resolve: (path: string) => string;
}
