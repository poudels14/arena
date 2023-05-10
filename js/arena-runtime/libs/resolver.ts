const { ops } = Arena.core;

class Resolver {
  #rid;
  root;

  constructor(options) {
    const [rid, root] = ops.op_resolver_new(options || {});
    this.#rid = rid;
    this.root = root;
  }

  resolve(specifier, referrer) {
    return ops.op_resolver_resolve(this.#rid, specifier, referrer);
  }
}

export { Resolver };
