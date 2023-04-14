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
    ops: {
      op_node_create_hash: (algorithm: string) => number;
      op_node_hash_update: (ctx: number, data: any) => boolean;
      op_node_hash_update_str: (ctx: number, data: any) => boolean;
      op_node_hash_digest: (ctx: number) => number[];
      op_node_hash_digest_hex: (ctx: number) => string;
    };
    opAsync: typeof OpAsync;
  }

  function readFile(path: string): Promise<Uint16Array>;
  function readFile(path: string, encoding?: "utf8"): Promise<String>;
  interface FileSystem {
    // get absolute path to project root
    cwdSync: () => string;
    lstatSync: (file: string) => Record<string, any>;
    realpathSync: (file: string) => string;
    readdirSync: (file: string) => string[];
    existsSync: (pathh: string) => boolean;
    mkdirSync: (path: string, options: { recursive: boolean }) => void;
    readFileSync: (pathh: string) => Uint16Array;
    readFile: typeof readFile;
    readToString: (path: string) => Promise<string>;
    readAsJson: (path: string) => Promise<string>;
    writeFileSync: (path: string, data: any) => void;
  }

  export type ResolverConfig = {
    preserve_symlink?: boolean;

    alias?: Record<string, string>;

    conditions?: string[];

    dedupe?: string[];
  };

  class Resolver {
    /**
     * Project root
     *
     * All resolved paths are relative to this path
     */
    root: string;

    /**
     * Returns a resolved path of the specifier relative
     * to the project root, which is same as {@link root}
     */
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
    root: string;

    public transpileFileAsync: (filename: string) => Promise<TranspileResult>;

    /**
     * If import resolution is enabled, the filename should be passed such that
     * the imports are resolved using the filename as a referrer
     */
    public transpileSync: (code: string, filename?: string) => TranspileResult;
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
  let fs: FileSystem;
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

declare function encodeToBase64(digest: any): string;
declare function encodeToBase64Url(digetst: any): string;

declare module "@arena/babel" {
  export const babel;
  export const solidPreset;
  export const transformCommonJsPlugin;
  export const importResolverPlugin;
}

declare module "@arena/rollup" {
  export const rollup;
  export const plugins: {
    terser: () => any;
    arenaResolver: (options: Arena.ResolverConfig) => any;
    arenaLoader: (options: {
      replace?: Arena.TranspilerConfig["replace"];
    }) => any;
    babel: (options: any) => any;
    postcss: (options: any) => any;
  };
  export const build: (options: any) => Promise<void>;
}

/**
 * Following node modules are only accessible when node modules are enabled
 */

declare var path: any;
declare var process: any;
