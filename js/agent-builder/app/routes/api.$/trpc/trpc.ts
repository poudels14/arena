import { initTRPC } from "@trpc/server";
import * as trpcExpress from "@trpc/server/adapters/express";

const createContext = ({
  req,
  res,
}: trpcExpress.CreateExpressContextOptions) => ({});
type Context = Awaited<ReturnType<typeof createContext>>;

const t = initTRPC.create<Context>();

const router = t.router;
const publicProcedure = t.procedure;
export { router, publicProcedure, createContext };
