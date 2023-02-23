declare namespace Arena {
  type Core = {
    ops: {
      /**
       * Receive a HTTP request.
       *
       * Only to be used by Arena Workspace Server!
       */
      op_receive_request: () => Promise<{ rid: number; internal: Request }>;

      /**
       * Send a response to the HTTP request
       *
       * Only to be used by Arena Workspace Server!
       */
      op_send_response: (
        rid: number,
        status: number,
        headers: [string, string][],
        data?: null | string | number
      ) => Promise<void>;

      op_read_file_sync: (filename: string) => Uint16Array;
      op_read_file_async: (filename: string) => Promise<Uint16Array>;
      op_read_file_string_async: (filename: string) => Promise<String>;
    };
  };

  type Env = Record<string, any>;

  type TranspileOptions = {
    source_map?: "inline";
  };

  type TranspileResult = {
    code: string;
  };

  type BuildTools = {
    babel: any;
    babelPlugins: any;
    babelPresets: {
      solid: any;
    };

    Transpiler: {
      transpileFileAsync: (
        filename: string,
        options: TranspileOptions
      ) => Promise<TranspileResult>;
      transpileSync: (
        code: string,
        options: TranspileOptions
      ) => TranspileResult;
    };
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
