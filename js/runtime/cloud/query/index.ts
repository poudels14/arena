declare var Arena;

type DataQueryTranspileResult = {
  errors: any[];
  propsGenerator: string;
  serverModule: string;
  /**
   * list of ids of the resources used by this data query
   */
  resources: string[];
};

const transpileDataQuery = async (
  code: string
): Promise<DataQueryTranspileResult> => {
  const { errors, propsGenerator, serverModule, resources } =
    await Arena.core.opAsync("op_cloud_transpile_js_data_query", code);

  return { errors, propsGenerator, serverModule, resources };
};

export { transpileDataQuery };
