import { Resolver } from "@arena/runtime/resolver";

const { core } = Arena;

const resolver = new Resolver({
  preserveSymlink: true,
});

const moduleCache = {};
const wrapper = [
  "(function (exports, require, module, __filename, __dirname) { (function _commonJs(exports, require, module, __filename, __dirname) {",
  "\n}).call(this, exports, require, module, __filename, __dirname); })",
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
    try {
      const resolved = resolver.resolve(args[0], referrer, "Require");
      return path.join(resolver.root, resolved);
    } catch (e) {
      console.error("Error resolving path [", args[0], "]:", e);
      return args[0];
    }
  }

  function require(modulePath, ...args) {
    let resolvedRequirePath = resolve(modulePath);

    const url = new URL("file://" + resolvedRequirePath);
    const resolvedUrl = url.toString();
    const resolvedPath = url.pathname;

    if (moduleCache[resolvedPath]) {
      return moduleCache[resolvedPath].exports;
    }
    let moduleCode = core.ops.op_resolver_read_file(resolvedPath);
    if (moduleCode) {
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

      moduleCache[resolvedPath] = mod;
      return mod.exports;
    }
    throw new Error('Error loading "' + args[0] + '"');
  }

  Object.assign(require, {
    resolve,
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
