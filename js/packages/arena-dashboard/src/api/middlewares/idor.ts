import { TRPCError } from "@trpc/server";
import { Context } from "../context";
import { AccessType } from "../auth";
import { WorkspaceAccessType } from "../auth/acl";

const checkWorkspaceAccess =
  <Input, Next extends Function>(
    fn: (input: Input) => string,
    access: WorkspaceAccessType
  ) =>
  async ({ ctx, input, next }: { ctx: Context; input: Input; next: Next }) => {
    if (!(await ctx.acl.hasWorkspaceAccess(fn(input), access))) {
      throw new TRPCError({
        code: "FORBIDDEN",
        message: "Access denied",
      });
    }
    return next({
      ctx,
    });
  };

const checkAppAccess =
  <Input, Next extends Function>(
    fn: (input: Input) => string,
    access: AccessType
  ) =>
  async ({ ctx, input, next }: { ctx: Context; input: Input; next: Next }) => {
    if (!(await ctx.acl.hasAppAccess(fn(input), access))) {
      throw new TRPCError({
        code: "FORBIDDEN",
        message: "Access denied",
      });
    }
    return next({
      ctx,
    });
  };

const checkResourceAccess =
  <Input, Next extends Function>(
    fn: (input: Input) => string,
    access: AccessType
  ) =>
  async ({ ctx, input, next }: { ctx: Context; input: Input; next: Next }) => {
    if (!(await ctx.acl.hasResourceAccess(fn(input), access))) {
      throw new TRPCError({
        code: "FORBIDDEN",
        message: "Access denied",
      });
    }
    return next({
      ctx,
    });
  };

export { checkWorkspaceAccess, checkAppAccess, checkResourceAccess };
