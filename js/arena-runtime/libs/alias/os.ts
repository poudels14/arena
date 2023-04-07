export default new Proxy(
  {},
  {
    get(_target, _prop) {
      throw new Error("'os' module not supported");
    },
  }
);
