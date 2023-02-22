interface FetchEvent {
  request: Request;
  env: Arena.Env;
}

type RequestContext = {
  path: string;
  query: unknown;
};

interface PageEvent extends FetchEvent {
  ctx: RequestContext;
}

export type { PageEvent };
