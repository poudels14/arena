interface FetchEvent {
  request: Request;
  env: Arena.Env;
  tags: any[];
}

type RequestContext = {
  path: string;
  query: unknown;
};

interface PageEvent extends FetchEvent {
  ctx: RequestContext;
}

export type { PageEvent };
