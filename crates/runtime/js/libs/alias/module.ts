export default new Proxy(
  {},
  {
    get(_target, _prop) {
      throw new Error("'module' module not supported");
    },
  }
);
