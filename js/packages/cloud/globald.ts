declare namespace Arena {
  interface Core {
    ops: Record<string, Function>;
    opAsync: Function;
  }

  let core: Core;
}
