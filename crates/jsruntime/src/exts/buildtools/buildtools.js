import {
  babel,
  solidPreset,
  transformCommonJsPlugin,
  importResolverPlugin,
} from "builtin:///@arena/babel";

((global) => {
  function require(...args) {
    throw new Error("require(...) not yet supported; args =" + args);
  }

  /**
   * Note(sagar): since Arena.BuildTools is set during runtime but this
   * modules is loaded during snapshotting, lazy load `resolver`
   */
  const { Resolver } = Arena.BuildTools;
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

  if (!global.Arena.BuildTools) {
    throw new Error("Arena.BuildTools is undefined");
  }

  /**
   * Note(sagar): support basic `require(...)` when buildtools are enabled such
   * that modules that use require(...) work
   */
  Object.assign(global, {
    require,
  });

  Object.assign(global.Arena.BuildTools, {
    babel,
    babelPresets: {
      solid: solidPreset,
    },
    babelPlugins: {
      transformCommonJs: transformCommonJsPlugin,
      importResolver: importResolverPlugin,
    },
  });
})(globalThis);
