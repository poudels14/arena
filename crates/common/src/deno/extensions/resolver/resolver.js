import { Resolver } from "@arena/runtime/resolver";

function require(...args) {
  throw new Error("require(...) not yet supported; args =" + args);
}

const resolver = new Resolver({
  preserve_symlink: true,
});

Object.assign(require, {
  resolve(...args) {
    if (args.length > 1) {
      throw new Error(
        "Only one argument to require.resolve(...) supported, passed:" + args
      );
    }
    try {
      const resolved = resolver.resolve(args[0], "./");
      return path.join(resolver.root, resolved);
    } catch (e) {
      return args[0];
    }
  },
});

/**
 * Note(sagar): support basic `require(...)` when resolver is enabled
 */
Object.assign(globalThis, {
  require,
});
