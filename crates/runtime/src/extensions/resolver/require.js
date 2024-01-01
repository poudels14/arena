import { Resolver } from "@arena/runtime/resolver";

const { core } = Arena;

const resolver = new Resolver({
  preserveSymlink: true,
});

const NODE_INTERNALS = [
  "fs",
  "constants",
  "path",
  "crypto",
  "tty",
  "url",
  "util",
  "os",
  "stream",
  "module",
];

const requireCache = {};
const wrapper = [
  "(function (exports, require, module, __filename, __dirname) { (function _commonJs(exports, require, module, __filename, __dirname, global) {",
  "\n}).call(this, exports, require, module, __filename, __dirname, globalThis); })",
];

// A very simple and super hacky support for require to make libraries like
// React work. This should only be used in development mode.
// THIS IS NOT MEANT TO BE USED IN PRODUCTION RUNTIME
function createRequire(referrer) {
  function resolve(...args) {
    if (args.length > 1) {
      throw new Error(
        "Only one argument to require.resolve(...) supported, passed:" + args
      );
    }

    const resolved = resolver.resolve(args[0], referrer, "Require");
    return path.join(resolver.root, resolved);
  }

  function require(modulePath, ...args) {
    let resolvedPath;
    let moduleCode;

    // check if it's internal path
    // strip "node:" prefix if there's any
    let isNodeInternal =
      NODE_INTERNALS.indexOf(modulePath.replace(/^node:/, "")) >= 0;
    if (isNodeInternal) {
      modulePath = modulePath.replace(/^node:/, "");
      resolvedPath = "node/" + modulePath;
    } else {
      // If error resolving, return undefined
      try {
        let resolvedRequirePath = resolve(modulePath);
        const url = new URL("file://" + resolvedRequirePath);
        resolvedPath = url.pathname;
      } catch (e) {
        return undefined;
      }
    }

    if (requireCache[resolvedPath]) {
      return requireCache[resolvedPath].exports;
    }

    // Set the cache before loading the code such that if there's a circular
    // dependency, a reference to the module is returned before the module
    // is loaded. This prevents infinite loop
    requireCache[resolvedPath] = { exports: {} };

    if (isNodeInternal) {
      moduleCode = `module.exports = Arena.__nodeInternal["${modulePath}"]`;
    } else {
      moduleCode = core.ops.op_resolver_read_file(resolvedPath);
    }

    if (moduleCode) {
      const resolvedUrl = "file://" + resolvedPath;
      moduleCode = moduleCode.replace(/^#!.*?\n/, "");
      const wrappedModuldeCode = `${wrapper[0]}${moduleCode}${wrapper[1]}`;
      const [func, err] = core.evalContext(wrappedModuldeCode, resolvedUrl);
      if (err) {
        throw err.thrown;
      }

      const mod = { exports: {} };
      func(
        mod.exports,
        globalThis.__internalCreateRequire(resolvedUrl),
        mod,
        resolvedPath,
        path.dirname(resolvedPath)
      );

      requireCache[resolvedPath].exports = mod.exports;
      return requireCache[resolvedPath].exports;
    }
    throw new Error('Error loading "' + args[0] + '"');
  }

  Object.assign(require, {
    resolve,
    cache: requireCache,
  });

  return require;
}

/**
 * Note(sagar): add `__internalCreateRequire(...)` to global when resolver
 * extension is enabled
 */
Object.assign(globalThis, {
  __internalCreateRequire: createRequire,
});

Arena.__nodeInternal = {
  ...(Arena.__nodeInternal || {}),
  module: {
    createRequire,
  },
};
