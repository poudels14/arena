/**
 *
 * Query options
 * @typedef {Object<string, any>} QueryOptions
 * @property {boolean} camelCase Whether to update column names to camel case
 */

/**
 * Connection config
 * @typedef {Object} ConnectionConfig
 * @property {string} [path]
 * @property {i32} [flags]
 * @property {QueryOptions} [options]
 */

const Flags = {
  SQLITE_OPEN_READ_ONLY: 1,
  SQLITE_OPEN_READ_WRITE: 2,
  SQLITE_OPEN_CREATE: 4,
  SQLITE_OPEN_URI: 64,
  SQLITE_OPEN_NO_MUTEX: 32768,
  SQLITE_OPEN_NOFOLLOW: 0x0100_0000,
};

const { ops, opAsync } = Arena.core;
class Client {
  /**
   * @type {ConnectionConfig} config
   */
  config;
  rid;

  constructor(config) {
    this.config = config;
    this.rid = ops.op_sqlite_create_connection({
      ...this.config,
      options: {
        camelCase: false,
      },
    });
  }

  async connect() {}

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
      "op_sqlite_execute_query",
      this.rid,
      sql,
      params || [],
      options
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

export { Flags, Client };
