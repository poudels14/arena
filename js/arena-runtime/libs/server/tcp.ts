import { ArenaRequest } from "./request";

const { opAsync } = Arena.core;

class Server {
  private constructor() {}

  static async init() {
    await opAsync("op_http_listen");
    return new Server();
  }

  [Symbol.asyncIterator]() {
    return this;
  }

  async next() {
    try {
      const conn_id = await opAsync("op_http_accept");
      return { value: new TcpStream(conn_id), done: false };
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

class TcpStream {
  connection_id: number;

  constructor(connection_id: number) {
    this.connection_id = connection_id;
  }

  [Symbol.asyncIterator]() {
    return this;
  }

  async next() {
    try {
      const req = await opAsync("op_http_start", this.connection_id);
      if (req) {
        return { value: new ArenaRequest(req[0], req[1]), done: false };
      }
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

export { Server, TcpStream };
