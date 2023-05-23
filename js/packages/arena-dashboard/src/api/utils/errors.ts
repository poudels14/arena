import { TRPCError } from "@trpc/server";

export const notFound = (message?: string) => {
  throw new TRPCError({
    code: "NOT_FOUND",
    message: message ?? "Not found",
  });
};
