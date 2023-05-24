import { TRPCError } from "@trpc/server";

export const notFound = (message?: string) => {
  throw new TRPCError({
    code: "NOT_FOUND",
    message: message ?? "Not found",
  });
};

export const badRequest = (message?: string) => {
  throw new TRPCError({
    code: "BAD_REQUEST",
    message: message ?? "400 Bad request",
  });
};
