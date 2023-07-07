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

const { ops, opAsync } = Arena.core;
class Client {
  /**
   * @type {ConnectionConfig} config
   */
  config;
  rid;

  constructor(config) {
    this.config = config;
  }

  async connect() {
    const rid = await opAsync("op_postgres_create_connection", this.config);
    this.rid = rid;
  }

  isConnected() {
    return ops.op_postgres_is_connected(this.rid);
  }

  async query(query, params, options) {
    if (this.rid == undefined) {
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

    const { rows, columns } = await opAsync(
      "op_postgres_execute_query",
      this.rid,
      sql,
      params || [],
      options || {
        camelCase: true,
      }
    );

    let cols = columns.values;
    const mappedRows = rows.map((r) => {
      return cols.reduce((agg, c, i) => {
        agg[c] = r[i];
        return agg;
      }, {});
    });

    return {
      rows: mappedRows,
    };
  }
}

export { Client };
