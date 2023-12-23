import { procedure } from "@arena/core/router";
import { Context } from "~/api/procedure";

type AssistantContext = Context & {};

const p = procedure<AssistantContext>().use(async ({ ctx, next }) => {
  return await next({
    ctx,
  });
  // TODO(sagar): do auth
});

export { p };
export type { AssistantContext };
