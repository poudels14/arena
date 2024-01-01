const { ops } = Arena.core;

// Note: keep this in sync with @portal/server-dev Resolver
class Resolver {
  #rid;
  root;

  constructor(options) {
    const [rid, root] = ops.op_resolver_new(options || {});
    this.#rid = rid;
    this.root = root;
  }

  resolve(specifier, referrer, resolutionType?: "Require" | "Import") {
    return ops.op_resolver_resolve(
      this.#rid,
      specifier,
      referrer,
      resolutionType
    );
  }
}

Arena.__arenaRuntime = {
  ...(Arena.__arenaRuntime || {}),
  "@arena/runtime/resolver": { Resolver },
};

export { Resolver };
