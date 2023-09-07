(() => {
  // TODO(sagar): combine this with moduleloader EnvSecret
  class EnvironmentSecret {
    constructor(id) {
      this.id = id;
      this.__type__ = "secret";
      Object.freeze(this);
    }
  }

  const env = Arena.core.ops.op_apps_load_env().reduce((acc, cur) => {
    const value = cur.isSecret
      ? new EnvironmentSecret(cur.secretId)
      : cur.value;

    // override app template's env by app's env if the key match
    if (acc[cur.key]) {
      if (cur.app?.id) {
        acc[cur.key] = value;
      }
    } else {
      acc[cur.key] = value;
    }
    return acc;
  }, {});
  Object.assign(globalThis.process.env, env);
})();
