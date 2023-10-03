import { TRPCError } from "@trpc/server";
import { Context } from "../context";

const useLoggedInMiddleware = (t: any) =>
  t.middleware(async ({ ctx, next }: { ctx: Context; next: any }) => {
    if (!ctx.user) {
      throw new TRPCError({
        code: "UNAUTHORIZED",
        message: "User not logged in",
      });
    }

    if (ctx.user.config?.waitlisted) {
      throw new TRPCError({
        code: "FORBIDDEN",
        cause: {
          waitlisted: true,
        },
      });
    }

    return next({ ctx });
  });

export { useLoggedInMiddleware };
