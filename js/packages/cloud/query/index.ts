const transpileServerFunction = async (code: string) => {
  // @ts-expect-error
  const [propsGenerator, serverModule] =
    // @ts-expect-error
    await Arena.core.opAsync("op_cloud_transpile_js_data_query", code);

  return {
    // TODO(sp): set parsing error
    errors: null,
    propsGenerator,
    serverModule,
  };
};

export { transpileServerFunction };
