interface FetchEvent {
  request: Request;
  cookies: Record<string, string>;
  env: any;
}

type RequestContext = {
  path: string;

  search: string;

  /**
   * Parsed {@link RequestContext.search}
   */
  query: Record<string, string>;
};

interface PageEvent extends FetchEvent {
  ctx: RequestContext;
  tags: any[];
}

export type { FetchEvent, PageEvent };
