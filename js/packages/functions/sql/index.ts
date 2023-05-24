type SqlDataQueryConfig = {
  /**
   * Database connection string
   */
  connectionString: string;

  /**
   * Whether to use the connection pool
   *
   * If not set, the connection will be initiated before executing the query
   * and termiated after the query is completed
   */
  pool?: number;

  query: {
    /**
     * Raw query string
     */
    sql: string;
    /**
     * Query parameters
     */
    params: any[];
  };
};

export type { SqlDataQueryConfig };
