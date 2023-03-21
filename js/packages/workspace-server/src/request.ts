const RESOLVE = Symbol("_request_resolver_");
const { opAsync } = Arena.core;

class ArenaRequest extends Request {
  rid: number;

  constructor(internal: any, response_rid: number) {
    super(internal.url, {
      headers: internal.headers,
    });
    this.rid = response_rid;
  }

  async [RESOLVE](asyncFn: any) {
    try {
      let res = await asyncFn();
      if (res instanceof Response) {
        this.responseWith(res);
      } else {
        // If the result isn't of type Response, respond with it's string
        // value
        await opAsync("op_send_response", this.rid, 200, [], String(res));
      }
    } catch (e) {
      console.error(e);
      await opAsync(
        "op_send_response",
        this.rid,
        500,
        [],
        "Internal Server Error"
      );
    }
  }

  async responseWith(response: any) {
    // TODO(sagar): consider not using Deno's Request/Response type, too
    // much going on there
    let innerResponse = Arena.toInnerResponse(response);

    let content =
      innerResponse.body?.streamOrStatic?.body || innerResponse.body?.source;
    // TODO(sagar): throw error if stream is used
    // if (content == undefined) {
    //   throw new Error("Stream response not supported!");
    // }
    await opAsync(
      "op_send_response",
      this.rid,
      innerResponse.status,
      innerResponse.headerList || [],
      content
    );
  }
}

export { ArenaRequest, RESOLVE };
