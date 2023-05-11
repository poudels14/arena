import { fetchRequestHandler } from "@trpc/server/adapters/fetch";
import { sqlQuerySourceConfigSchema } from "@arena/appkit/widget/types/data";
import { z } from "zod";
import { createContext } from "./context";
import { procedure, router as trpcRouter } from "./trpc";

const r = trpcRouter({
  healthy: procedure.query(() => {
    return "OK";
  }),
  execSql: procedure.input(z.any()).query(({ input }) => {
    console.log("input =", input);

    // import env from "@appkit/env";
    // import handler from "@appkit/widgets/68L2YtU8r6Gb8NYN2MPJLu/rows";
    // console.log('yoooo, this is awesome');
    // import query from "@appkit/widgets/"
    // Sample SQL QUERY
    // `
    // SELECT * FROM users where name = {{ name }}
    // `;

    import("@appkit/widgets/widget_id/rows").then((m) => {
      console.log("imported module =", m);
    });

    return "Noice";
  }),
});

type RouterConfig = {
  workspaceId: string;
};

const router = (config: RouterConfig) => {
  return {
    route: async (request: Request) => {
      return await fetchRequestHandler({
        endpoint: "",
        req: request,
        router: r,
        createContext,
      });
    },
  };
};

export type DqsRouter = typeof r;
export { router };
