declare namespace Arena {
  type Core = {
    ops: Record<string, any>;
  };

  type Env = Record<string, any>;

  let core: Core;
  let env: Env;
  let BuildTools: any;
  let wasi: any;

  let toInnerResponse: (response: Response) => any;
}
