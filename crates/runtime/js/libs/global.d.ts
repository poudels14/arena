// WARNING: keep this in sync with js/runtime/core/global.d.ts

/**
 * Following node modules are only accessible when node modules are enabled
 */
declare var path: any;
declare var process: any;
declare var Buffer: any;

declare module "node:path";
declare module "node:crypto";

declare namespace Arena {
  var fs;
  var core;
  // This is used to expose builtin node modules that can be loaded
  // from "require(...)"
  var __nodeInternal;
  // This is used to expose @arena/runtime modules that can be loaded
  // from "require(...)"
  var __arenaRuntime: {
    "@arena/runtime/resolver": any;
    "@arena/runtime/transpiler": any;
  };

  export type ResolverConfig = {
    preserveSymlink?: boolean;

    alias?: Record<string, string>;

    conditions?: string[];

    dedupe?: string[];

    external?: string[];
  };

  export type TranspilerConfig = {
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
}

declare module "@arena/runtime/resolver";
declare module "@arena/runtime/transpiler" {
  var Transpiler;
}
declare module "@arena/runtime/babel";
