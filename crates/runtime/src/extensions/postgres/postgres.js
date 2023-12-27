/**
 *
 * Query options
 * @typedef {Object<string, any>} QueryOptions
 * @property {boolean} camelCase Whether to update column names to camel case
 */

/**
 * Connection config
 * @typedef {Object} ConnectionConfig
 * @property {string} [credential]
 * @property {QueryOptions} [options]
 */

function Field(obj) {
  const self = this;
  Object.entries(obj).forEach(([key, value]) => {
    self[key] = value;
  });
}

const { ops, opAsync } = Arena.core;
class Client {
  /**
   * @type {ConnectionConfig} config
   */
  #config;
  #rid;

  constructor(config) {
    this.#config = config;
  }

  async connect() {
    const rid = await opAsync("op_postgres_create_connection", this.#config);
    this.#rid = rid;
  }

  isConnected() {
    return this.#rid != undefined && ops.op_postgres_is_connected(this.#rid);
  }

  async query(query, params, options) {
    // reconnect if the connection was disconnected somehow
    if (!this.isConnected()) {
      await this.connect();
    }

    if (this.#rid == undefined) {
      throw new Error("Connection not initialized");
    }

    /**
     * Note(sagar): if teh result of solink sql`` is passed as {@link sql},
     * destructure query and parameters
     */
    let sql = query;
    if (typeof query === "object") {
      sql = query.sql;
      params = query.params;
    }

    const { rows, fields, ...rest } = await opAsync(
      "op_postgres_execute_query",
      this.#rid,
      sql,
      params,
      options || {
        camelCase: true,
      }
    );

    return {
      ...rest,
      // Use getter such that they rows are mapped lazily
      // This prevents having to map rows when using ORM like drizzle
      // that doesn't need mapping
      get rows() {
        return rows.map((row) => {
          return fields.reduce((agg, field, i) => {
            agg[field._casedName || field.name] = row[i];
            return agg;
          }, {});
        });
      },
      get fields() {
        fields.map((field) => {
          return new Field(field);
        });
      },
      _raw: {
        values: rows,
      },
    };
  }

  async transaction(closure) {
    await this.query("BEGIN");
    await Promise.resolve(closure())
      .then(async () => {
        await this.query("COMMIT");
      })
      .catch(async (e) => {
        await this.query("ROLLBACK");
        throw e;
      });
  }

  // Noop if the connection is already closed
  close() {
    if (this.#rid) {
      ops.op_postgres_close(this.#rid);
    }
  }

  // This is for drizzle support
  // Using this client with `drizzle` from "drizzle-orm/postgres-js"
  // should work out of the box
  unsafe(query, params) {
    const client = this;
    return {
      async values() {
        const res = await client.query(query, params);
        return res._raw.values;
      },
    };
  }
}

export { Client };
