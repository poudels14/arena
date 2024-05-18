import { createTRPCReact } from "@trpc/react-query";

import type { AppRouter } from "./routes/api.$/trpc/router";

export const trpc = createTRPCReact<AppRouter>();
