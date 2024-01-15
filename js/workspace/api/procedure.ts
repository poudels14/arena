import { procedure } from "@portal/server-core/router";

type Context = {
  user: any;
};

const p = procedure<Context>().use(async ({ ctx, next }) => {
  return await next({
    ctx,
  });
  // TODO(sagar): do auth
});

export { p };
export type { Context };
