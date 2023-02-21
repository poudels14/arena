declare namespace Arena {
  type Core = {
    ops: Record<string, any>;
  };

  let core: Core;
  let env: Record<string, any>;
  let BuildTools: any;
  let wasi: any;

  let toInnerResponse: (response: Response) => any;
}
