const { ops } = Arena.core;

class Resolver {
  #rid;
  root;

  constructor(options) {
    const [rid, root] = ops.op_resolver_new(options || {});
    this.#rid = rid;
    this.root = root;
  }

  resolve(specifier, referrer, resolutionType: "Require" | "Import") {
    return ops.op_resolver_resolve(
      this.#rid,
      specifier,
      referrer,
      resolutionType
    );
  }
}

export { Resolver };
