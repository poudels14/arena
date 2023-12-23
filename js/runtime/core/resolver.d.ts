declare module "@arena/runtime/resolver" {
  export class Resolver {
    constructor(config?: ResolverConfig);

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

    close(): void;
  }
}
