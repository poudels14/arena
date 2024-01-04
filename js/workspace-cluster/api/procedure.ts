import { procedure } from "@portal/server-core/router";

type Context = {
  user: any;
};

const p = procedure<Context>().use(async ({ ctx, next }) => {
  // TODO: auth
  return await next({
    ctx,
  });
});

export { p };
export type { Context };
