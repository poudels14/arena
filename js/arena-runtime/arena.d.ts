declare namespace Arena {
  type Env = {
    /**
     * This flag can be used to tree-shake client and server side code
     * branches depending on which env the bundle is being generated for
     */
    SSR: "true" | "false";
  } & Record<string, string>;

  /**
   * Start TCP server; Works only if the http server extension is used
   */
  function OpAsync(name: "op_http_listen"): Promise<void>;

  /**
   * Listen for new TCP connection
   */
  function OpAsync(name: "op_http_accept"): Promise<number | null>;

  /**
   * Receive a HTTP request
   */
  function OpAsync(
    name: "op_http_start",
    rid: number
  ): Promise<[rid: number, internal: Request] | null>;

  /**
   * Send a response to the HTTP request
   */
  function OpAsync(
    fn: "op_http_send_response",
    rid: number,
    status: number,
    headers: [string, string][],
    data: null | string | number,
    isStream: boolean | null | undefined
  ): Promise<[number | null, number | null, any]>;

  /**
   * Write data to stream if the streaming response is being sent
   * This is used for SSE, etc
   *
   * Returns the length of bytes written to the stream
   * Returns -1 if write failed
   */
  function OpAsync(
    fn: "op_http_write_data_to_stream",
    streamId: number,
    eventName: "data",
    data: any
  ): Promise<void>;

  function OpAsync(
    fn: "op_websocket_recv",
    rxId: number,
    txId: number
  ): Promise<any>;

  /**
   * returns 0 if sending message failed
   */
  function OpAsync(
    fn: "op_websocket_send",
    txId: number,
    data: any
  ): Promise<number>;

  function OpAsync(
    fn: "op_websocket_recv",
    rxId: number,
    txId: number
  ): Promise<any>;

  /**
   * Transpile the given filename
   */
  function OpAsync(
    name: "op_transpiler_transpile_file_async",
    rid: number,
    filename: string
  ): Promise<string>;

  /**
   * Start TCP server on given address and port
   *
   * Only available when dqs extension is used
   */
  function OpAsync(
    name: "op_dqs_start_tcp_server",
    workspaceId: string,
    address: string,
    port: number
  ): Promise<number>;

  /**
   * Start mpsc stream server
   *
   * Only available when dqs extension is used
   */
  function OpAsync(
    name: "op_dqs_start_stream_server",
    workspaceId: string
  ): Promise<[number, number]>;

  /**
   * Returns a list of resource ids of all active server's handle
   */
  function OpAsync(name: "op_dqs_list_servers"): Promise<number[]>;

  /**
   * Ping the DQS server to see if the server thread is running
   */
  function OpAsync(name: "op_dqs_ping", handleId: number): Promise<"PONG">;

  /**
   * Terminate the server corresponding to the given handle id
   */
  function OpAsync(
    name: "op_dqs_terminate_server",
    handleId: number
  ): Promise<void>;

  /**
   * Send request to mpsc stream server and return response
   */
  function OpAsync(
    name: "op_dqs_pipe_request_to_stream",
    streamId: number,
    request: [
      url: string,
      method: Request["method"],
      headers: [string, string][],
      body?: any
    ]
  ): Promise<[number, [string, string][], any]>;

  interface Core {
    ops: {
      op_node_create_hash: (algorithm: string) => number;
      op_node_hash_update: (ctx: number, data: any) => boolean;
      op_node_hash_update_str: (ctx: number, data: any) => boolean;
      op_node_hash_digest: (ctx: number) => number[];
      op_node_hash_digest_hex: (ctx: number) => string;
      op_node_generate_secret: (buffer: any) => void;

      /**
       * Returns true if stream was successfully closed
       */
      op_http_close_stream: (id: number) => boolean;

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
      ) => string | null;

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

      /**
       * Check if the DQS server is alive
       *
       * Only available if DQS module is used
       */
      op_dqs_is_alive: (handleId: number) => boolean;
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
  /**
   * Arena config loaded from "package.json" 's "arena" field
   */
  let config: {
    name: string;
    version: string;
    env?: Record<string, string>;
    javascript?: {
      resolve?: ResolverConfig;
    };
    client?: Pick<typeof config, "env" | "javascript">;
    server?: Pick<typeof config, "env" | "javascript">;
  };
  let env: Env;
  let fs: FileSystem;
  let wasi: any;

  type ResolverConfig = {
    preserveSymlink?: boolean;

    alias?: Record<string, string>;

    conditions?: string[];

    dedupe?: string[];
  };

  type TranspilerConfig = {
    /**
     * Whether to resolve the import when transpiling
     */
    resolveImport?: boolean;

    resolver?: ResolverConfig;

    /**
     * A set of key/value that will be replaced
     * when transpiling. Works similar to @rollup/plugin-replace
     */
    replace?: Record<string, string>;

    sourceMap?: "inline";
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
  // @ts-expect-error
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

declare module "node:path";
declare module "node:crypto";

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
  type ClientConfig = {
    credential:
      | string
      | {
          host: string;
          port: string;
          username: string;
          password: string;
          database: string;
        };
  };

  type Client = {
    connect(): Promise<void>;
    isConnected(): boolean;

    query<T>(sql: string, parameters?: any[]): Promise<{ rows: T[] }>;
    query<T>(query: {
      sql: string;
      params: readonly any[];
    }): Promise<{ rows: T[] }>;
  };

  export const Client: new (config: ClientConfig) => Client;
}

declare module "@arena/runtime/sqlite" {
  export const Flags: {
    SQLITE_OPEN_READ_ONLY: 1;
    SQLITE_OPEN_READ_WRITE: 2;
    SQLITE_OPEN_CREATE: 4;
    SQLITE_OPEN_URI: 64;
    SQLITE_OPEN_NO_MUTEX: 32768;
    SQLITE_OPEN_NOFOLLOW: 0x0100_0000;
  };

  type ClientConfig = {
    path: String;
    flags?: number;
    options?: {
      camelCase?: boolean;
    };
  };

  type Client = {
    query<T>(sql: string, parameters?: any[]): Promise<{ rows: T[] }>;
    query<T>(query: {
      sql: string;
      params: readonly any[];
    }): Promise<{ rows: T[] }>;
    transaction<T>(closure: () => T | Promise<T>): Promise<void>;
    close(): Promise<void>;
  };

  export const Client: new (config: ClientConfig) => Client;
}

declare module "@arena/runtime/server" {
  interface Websocket extends AsyncIterator<any> {
    /**
     * returns 0 if sending message failed
     */
    send(data: any): Promise<number>;
    close(data?: any): Promise<void>;
    next(): Promise<any>;
  }

  type ServeConfig = {
    fetch: (req: Request) => Promise<Response>;
    websocket?: (websocket: Websocket, data: any) => void;
  };

  type Handler<Context> = {
    (args: { req: Request; ctx: Context }): Promise<Response>;
    method: "GET" | "POST" | "PATCH";
  };

  type RouterConfig<Context> = {
    host?: string;
    prefix?: string;
    defaultHandler?: ProcedureCallback<Context>;

    /**
     * A middleware similar to procedure middleware but applies to
     * all routes under this router. This can be used to setup higher level
     * middleware if the router doesn't have access to route level procedures
     * or has sub-routers with different procedures
     */
    middleware?: ProcedureCallback<Context>;
  };

  type ProcedureCallbackArgs<Context> = {
    req: Request;
    env: any;
    ctx: Context;
    params: Record<string, string>;
    searchParams: Record<string, string>;
    cookies: Record<string, string>;
    errors: {
      notFound(message?: string): void;
      badRequest(message?: string): void;
      forbidden(message?: string): void;
      internalServerError(message?: string): void;
    };
    setHeader: (name: string, value: string) => void;
    setCookie(
      name: string,
      value: string,
      options?: {
        domain?: string | undefined;
        /**
         * The default function is the global `encodeURIComponent`
         */
        encode?(value: string): string;
        /**
         * By default, no expiration is set, and most clients will delete it after session expires
         */
        expires?: Date | undefined;
        /**
         * By default, the `HttpOnly` attribute is not set.
         */
        httpOnly?: boolean | undefined;
        maxAge?: number | undefined;
        /**
         * By default, the path is considered the "default path".
         */
        path?: string | undefined;
        priority?: "low" | "medium" | "high" | undefined;
        sameSite?: true | false | "lax" | "strict" | "none" | undefined;
        /**
         * By default, the `Secure` attribute is not set.
         */
        secure?: boolean | undefined;
      }
    ): void;
    clearCookie: (name: string) => void;
    form: {
      /**
       * Parse request for multipart form data
       */
      multipart: (req: Request) => Promise<
        {
          filename: string;
          type: string;
          name: string;
          data: Buffer;
        }[]
      >;
    };
    redirect: (path: string) => void;
    next: (args: Partial<ProcedureCallbackArgs<Context>>) => void;
  };

  type ProcedureCallback<Context> = (
    args: ProcedureCallbackArgs<Context>
  ) => Promise<any> | any;

  type Procedure<Context> = {
    use(cb: ProcedureCallback<Context>): Procedure<Context>;
    query(cb: ProcedureCallback<Context>): Handler<Context>;
    mutate(cb: ProcedureCallback<Context>): Handler<Context>;
    patch(cb: ProcedureCallback<Context>): Handler<Context>;
  };

  export const procedure: <Context>() => Procedure<Context>;

  export const createRouter: <Context>(
    config: RouterConfig<Context> & {
      routes: Record<string, Handler<Context>>;
    }
  ) => {
    route(
      request: Request,
      meta?: { env?: Record<string, string | undefined>; context?: Context }
    ): Promise<Response>;
  };

  export const mergedRouter: <Context>(
    config: RouterConfig<Context> & {
      prefix?: string;
      routers: ReturnType<typeof createRouter<Context>>[];
    }
  ) => ReturnType<typeof createRouter<Context>>;

  export const serve: (config: ServeConfig) => Promise<void>;
}

declare module "@arena/runtime/dqs" {
  export class DqsServer {
    // returns whether the DQS server is alive
    isAlive(): boolean;

    pipeRequest(request: {
      url: string;
      method?: Request["method"];
      headers?: [string, string][];
      body?: any;
    }): Promise<
      [
        // status code
        number,
        // headers
        [string, string][],
        // body
        any
      ]
    >;
  }

  export class DqsCluster {
    static startTcpServer(
      workspaceId: string,
      address: string,
      port: number
    ): Promise<DqsServer>;

    static startStreamServer(workspaceId: string): Promise<DqsServer>;
  }
}

declare module "@arena/runtime/bundler" {
  type BuildConfig = {
    env?: Record<string, any>;
    javascript?: {
      resolve?: {
        alias?: Record<string, string>;
        conditions?: string[];
        dedupe?: string[];
      };
    };
  };

  /**
   * Build server bundle
   */
  export const server: (options: {
    input: string;
    output: any;
    javascript?: BuildConfig["javascript"];
    /**
     * rollup plugins
     */
    plugins?: any[];
  }) => Promise<void>;

  /**
   * Build client bundle
   */
  export const client: (options: {
    input: string;
    output: any;
    env?: BuildConfig["env"];
    javascript?: BuildConfig["javascript"];
    /**
     * rollup plugins
     */
    plugins?: any[];
  }) => Promise<void>;
}

declare module "@arena/runtime/filerouter" {
  type FileLoaderOptions = {
    env?: Record<string, string>;
    resolve?: Arena.ResolverConfig;
  };

  export const createFileRouter: (
    options: FileLoaderOptions
  ) => (req: Request) => Promise<Response | undefined>;
}
