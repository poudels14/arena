import { initTRPC } from "@trpc/server";
import { procedure as createProcedure } from "@arena/runtime/server";
import type { Context } from "./context";
import { useLoggedInMiddleware } from "./middlewares";

/**
 * Initialization of tRPC backend
 * Should be done only once per backend!
 */
const t = initTRPC.context<Context>().create();

/**
 * Export reusable router and procedure helpers
 * that cane be used throughout the router
 */
const router = t.router;
const publicProcedure = t.procedure;
const procedure = t.procedure.use(useLoggedInMiddleware(t));

/**
 * This is for non-trpc routes
 */
const p = createProcedure<any>().use(() => {});

export { t, router, publicProcedure, procedure, p };
