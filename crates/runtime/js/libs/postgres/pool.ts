import { createPool, Pool as GenericPool } from "generic-pool";
import { Client, ConnectionConfig } from "./client";

type PoolOptions = {
  max?: number;
  min?: number;
};

class Pool {
  #pool: GenericPool<Client>;
  constructor(config: ConnectionConfig & PoolOptions) {
    const { max = 10, min = 1 } = config;
    this.#pool = createPool(
      {
        create: async function () {
          return new Client(config);
        },
        destroy: async function (client) {
          client.close();
        },
      },
      {
        max,
        min,
      }
    );
  }

  get size() {
    return this.#pool.size;
  }

  get available() {
    return this.#pool.available;
  }

  get borrowed() {
    return this.#pool.borrowed;
  }

  get pending() {
    return this.#pool.pending;
  }

  async connect(): Promise<Client & { release(): Promise<void> }> {
    const pool = this.#pool;
    const client = await pool.acquire();
    return Object.assign(client, {
      async release() {
        await pool.release(client);
      },
    });
  }

  async query(...args: [any]) {
    const client = await this.connect();
    const res = await client.query(...args);
    await client.release();
    return res;
  }
}

export { Pool };
