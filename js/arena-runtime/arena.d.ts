declare namespace Arena {
  type Env = {
    /**
     * This flag can be used to tree-shake client and server side code
     * branches depending on which env the bundle is being generated for
     */
    SSR: boolean;
  } & Record<string, any>;

  /**
   * Start TCP server; Works only if the http server extension is used
   */
  function OpAsync(name: "op_http_listen"): Promise<void>;

  /**
   * Listen for new TCP connection
   */
  function OpAsync(name: "op_http_accept"): Promise<number>;

  /**
   * Receive a HTTP request
   */
  function OpAsync(
    name: "op_http_start",
    rid: number
  ): Promise<[rid: number, internal: Request] | undefined>;

  /**
   * Send a response to the HTTP request
   */
  function OpAsync(
    fn: "op_http_send_response",
    rid: number,
    status: number,
    headers: [string, string][],
    data?: null | string | number
  ): Promise<void>;

  /**
   * Transpile the given filename
   */
  function OpAsync(
    name: "op_transpiler_transpile_file_async",
    rid: number,
    filename: string
  ): Promise<string>;

  interface Core {
    ops: {
      op_node_create_hash: (algorithm: string) => number;
      op_node_hash_update: (ctx: number, data: any) => boolean;
      op_node_hash_update_str: (ctx: number, data: any) => boolean;
      op_node_hash_digest: (ctx: number) => number[];
      op_node_hash_digest_hex: (ctx: number) => string;
      op_node_generate_secret: (buffer: any) => void;

      /**
       * Only set if Resolver module is used
       *
       * @param config
       * @returns [resource_id, root_dir]
       */
      op_resolver_new: (config: ResolverConfig) => [number, string];

      /**
       *
       * @param rid resource id of the resolver
       * @param specifier module specifier
       * @param referrer referrer
       * @returns resolved path of the specifier if found
       */
      op_resolver_resolve: (
        rid: number,
        specifier: string,
        referrer: string
      ) => string | undefined;

      /**
       * Only set if Transpiler module is used
       *
       * @param config
       * @returns [resource_id, root_dir]
       */
      op_transpiler_new: (config: TranspilerConfig) => [number, string];

      op_transpiler_transpile_sync: (
        rid: number,
        filename: string,
        code: string
      ) => string;
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

  let core: Core;
  let env: Env;
  let fs: FileSystem;
  let wasi: any;

  type ResolverConfig = {
    preserve_symlink?: boolean;

    alias?: Record<string, string>;

    conditions?: string[];

    dedupe?: string[];
  };

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

  // this should be exposed by runtime
  export type Response = globalThis.Response & {
    toInnerResponse: (response: globalThis.Response) => any;
  };
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

/**
 * Following node modules are only accessible when node modules are enabled
 */
declare var path: any;
declare var process: any;
declare var Buffer: any;

declare module "@arena/runtime/resolver" {
  export class Resolver {
    constructor(config?: Arena.ResolverConfig);

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
}

declare module "@arena/runtime/transpiler" {
  type TranspileResult = {
    code: string;
  };

  class Transpiler {
    root: string;

    constructor(config?: Arena.TranspilerConfig);

    public transpileFileAsync: (filename: string) => Promise<TranspileResult>;

    /**
     * If import resolution is enabled, the filename should be passed such that
     * the imports are resolved using the filename as a referrer
     */
    public transpileSync: (code: string, filename?: string) => TranspileResult;
  }
}

declare module "@arena/runtime/babel" {
  export const babel;
  export const presets: {
    solidjs;
  };
  export const plugins: {
    transformCommonJs;
    importResolver;
  };
}

declare module "@arena/runtime/rollup" {
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

declare module "@arena/runtime/postgres" {
  type ClientConfig =
    | {
        connectionStringId: number;
      }
    | {
        connectionString: string;
      };

  type Client = {
    connect(): Promise<void>;
    isConnected(): boolean;

    query<T>(sql: string, parameters?: any[]): Promise<{ rows: T[] }>;
    query<T>(query: {
      type: "SLONIK_TOKEN_SQL";
      sql: string;
      values: readonly any[];
    }): Promise<{ rows: T[] }>;
  };

  export const Client: new (config: ClientConfig) => Client;
}

declare module "@arena/runtime/server" {
  type ServeConfig = {
    fetch: (req: Request) => Promise<Response>;
  };

  export const serve: (config: ServeConfig) => Promise<void>;
}
