const { opAsync } = Arena.core;

class ArenaRequest extends Request {
  rid: number;

  constructor(rid: number, internal: any) {
    super(internal.url, internal);
    this.rid = rid;
  }

  public async send(response: Response) {
    // TODO(sagar): consider not using Deno's Request/Response type, too
    // much going on there
    let innerResponse = (Response as unknown as Arena.Response).toInnerResponse(
      response
    );

    let content =
      innerResponse.body?.streamOrStatic?.body || innerResponse.body?.source;
    // TODO(sagar): throw error if stream is used
    await opAsync(
      "op_http_send_response",
      this.rid,
      innerResponse.status,
      innerResponse.headerList || [],
      content
    );
  }
}

export { ArenaRequest };
