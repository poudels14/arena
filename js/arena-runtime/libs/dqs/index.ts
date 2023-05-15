class DqsServer {
  #handleId: number;
  /**
   * Only set for stream type server
   */
  #streamId: number | undefined;
  private constructor(handleId: number, streamId: number | undefined) {
    this.#handleId = handleId;
    this.#streamId = streamId;
  }

  static async startTcpServer(
    workspaceId: string,
    address: string,
    port: number
  ) {
    const handleId = await Arena.core.opAsync(
      "op_dqs_start_tcp_server",
      workspaceId,
      address,
      port
    );
    return new DqsServer(handleId, undefined);
  }

  static async startStreamServer(workspaceId: string) {
    const [handleId, streamId] = await Arena.core.opAsync(
      "op_dqs_start_stream_server",
      workspaceId
    );
    return new DqsServer(handleId, streamId);
  }

  // returns whether the DQS server is alive
  isAlive() {
    return Arena.core.ops.op_dqs_is_alive(this.#handleId);
  }

  async pipeRequest(request: Request) {
    if (this.#streamId !== undefined) {
      const response = await Arena.core.opAsync(
        "op_dqs_pipe_request_to_stream",
        this.#streamId,
        {
          url: request.url,
          method: request.method,
          headers: [],
          body: undefined,
        }
      );
      return response;
    } else {
      throw new Error("Can only pipe request to stream type DqsServer");
    }
  }
}

export { DqsServer };
