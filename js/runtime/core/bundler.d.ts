type BuildConfig = {
  env?: Record<string, any>;
  javascript?: {
    resolve?: {
      alias?: Record<string, string>;
      conditions?: string[];
      dedupe?: string[];
    };
  };
};

/**
 * Build server bundle
 */
export const server: (options: {
  input: string;
  output: any;
  javascript?: BuildConfig["javascript"];
  /**
   * rollup plugins
   */
  plugins?: any[];
}) => Promise<void>;

/**
 * Build client bundle
 */
export const client: (options: {
  input: string;
  output: any;
  env?: BuildConfig["env"];
  javascript?: BuildConfig["javascript"];
  /**
   * rollup plugins
   */
  plugins?: any[];
}) => Promise<void>;
