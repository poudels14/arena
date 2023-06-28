import { Websocket } from "./websocket";

const { ops, opAsync } = Arena.core;
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

    if (innerResponse?.body?.streamOrStatic instanceof ReadableStream) {
      const body = innerResponse?.body?.streamOrStatic;
      const [writerId] = await opAsync(
        "op_http_send_response",
        this.rid,
        innerResponse.status,
        innerResponse.headerList || [],
        null,
        true
      );

      // TODO(sagar): handle async/await subscription better
      let next;
      let reader = body.getReader();
      while ((next = await reader.read()) && !next.done) {
        await opAsync(
          "op_http_write_data_to_stream",
          writerId!,
          "data",
          next.value
        );
      }
      ops.op_http_close_stream(writerId!);
      return;
    }

    let content =
      innerResponse.body?.streamOrStatic?.body || innerResponse.body?.source;
    // TODO(sagar): throw error if stream is used
    let maybeWebsocket = await opAsync(
      "op_http_send_response",
      this.rid,
      innerResponse.status,
      innerResponse.headerList || [],
      content,
      false
    );

    if (maybeWebsocket) {
      return [
        new Websocket(maybeWebsocket[0], maybeWebsocket[1]),
        maybeWebsocket[2],
      ];
    }
    return undefined;
  }
}

export { ArenaRequest };
