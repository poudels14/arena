import { fetchRequestHandler } from "@trpc/server/adapters/fetch";
import { ActionFunctionArgs, LoaderFunctionArgs } from "@remix-run/node";

import { createContext } from "./trpc/trpc";
import { appRouter } from "./trpc/router";
import { router } from "./routes";

export const loader = async (args: LoaderFunctionArgs) => {
  return handleRequest(args);
};

export const action = async (args: ActionFunctionArgs) => {
  return handleRequest(args);
};

async function handleRequest(args: LoaderFunctionArgs | ActionFunctionArgs) {
  const res = await router.route(args.request);
  if (res) {
    return res;
  }

  return fetchRequestHandler({
    endpoint: "/api",
    req: args.request,
    router: appRouter,
    // @ts-expect-error
    createContext,
  });
}
