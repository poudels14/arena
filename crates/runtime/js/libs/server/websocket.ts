const { opAsync } = Arena.core;

class Websocket {
  #rxId: number;
  #txId: number;

  constructor(rxId, txId) {
    this.#rxId = rxId;
    this.#txId = txId;
  }

  async send(data: any) {
    return await opAsync("op_websocket_send", this.#txId, {
      close: false,
      payload: data,
    });
  }

  async close(data?: any) {
    return await opAsync("op_websocket_send", this.#txId, {
      close: true,
      payload: data,
    });
  }

  [Symbol.asyncIterator]() {
    return this;
  }

  async next() {
    try {
      const value = await opAsync("op_websocket_recv", this.#rxId, this.#txId);
      return { value, done: value?.close };
    } catch (error) {
      console.error(
        new Error("[runtime error]", {
          cause: error,
        })
      );
    }
    return { value: undefined, done: true };
  }
}

export { Websocket };
