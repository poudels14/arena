"use strict";
((global) => {
  const { ops } = Arena.core;

  class Resolver {
    #rid;
    
    constructor(options) {
      const rid = ops.op_resolver_new(options || {});
      this.#rid = rid;
    }

    resolve(specifier, referrer) {
      return ops.op_resolver_resolve(this.#rid, specifier, referrer);
    }
  }

  Object.assign(global.Arena.BuildTools, {
    Resolver,
  });
})(globalThis);
