declare namespace Arena {
  type Env = {
    /**
     * This flag can be used to tree-shake client and server side code
     * branches depending on which env the bundle is being generated for
     */
    SSR: boolean;
  } & Record<string, any>;

  /**
   * Receive a HTTP request.
   *
   * Only to be used by Arena Workspace Server!
   */
  function OpAsync(
    name: "op_receive_request"
  ): Promise<{ rid: number; internal: Request }>;

  /**
   * Send a response to the HTTP request
   *
   * Only to be used by Arena Workspace Server!
   */
  function OpAsync(
    fn: "op_send_response",
    rid: number,
    status: number,
    headers: [string, string][],
    data?: null | string | number
  ): Promise<void>;

  interface Core {
    ops: {};
    opAsync: typeof OpAsync;
  }

  type ResolverConfig = {
    preserve_symlink?: boolean;

    alias?: Record<string, string>;

    conditions?: string[];

    dedupe?: string[];
  };

  class Resolver {
    resolve(specifier: string, referrer: string): string;

    close();
  }

  type TranspilerConfig = {
    /**
     * Whether to resolve the import when transpiling
     */
    resolve_import?: boolean;

    resolver?: ResolverConfig;

    /**
     * A set of key/value that will be replaced
     * when transpiling. Works similar to @rollup/plugin-replace
     */
    replace?: Record<string, string>;

    source_map?: "inline";
  };

  type TranspileResult = {
    code: string;
  };

  class Transpiler {
    public transpileFileAsync: (filename: string) => Promise<TranspileResult>;

    public transpileSync: (code: string) => TranspileResult;
  }

  type BuildTools = {
    babel: any;
    babelPlugins: {
      transformCommonJs: any;
      importResolver: any;
    };
    babelPresets: {
      solid: any;
    };

    Transpiler: new (config?: TranspilerConfig) => Transpiler;

    Resolver: new (config?: ResolverConfig) => Resolver;
  };

  let core: Core;
  let env: Env;
  let fs: {
    existsSync: (path) => boolean;
    readFileSync: (path) => Uint16Array;
    readFile: (path) => Promise<Uint16Array>;
    readToString: (path) => Promise<string>;
  };
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
