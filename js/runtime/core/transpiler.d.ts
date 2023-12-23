type TranspileResult = {
  code: string;
};

export class Transpiler {
  root: string;

  constructor(config?: TranspilerConfig);

  public transpileFileAsync: (filename: string) => Promise<TranspileResult>;

  /**
   * If import resolution is enabled, the filename should be passed such that
   * the imports are resolved using the filename as a referrer
   */
  public transpileSync: (code: string, filename?: string) => TranspileResult;
}
