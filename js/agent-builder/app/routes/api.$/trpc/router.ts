import { router } from "./trpc";
import { getAgent, updateAgent } from "./agent";

const appRouter = router({
  getAgent,
  updateAgent,
});

export type AppRouter = typeof appRouter;
export { appRouter };
