import { minify } from "terser";

const terser = () => {
  return {
    name: "arena-terser",
    async renderChunk(code, chunk, options) {
      options = {
        sourceMap:
          options.sourcemap === true || typeof options.sourcemap === "string",
        module: options.format === "es",
        toplevel: options.format === "cjs",
      };

      const result = await minify(code, options);
      const output = {
        code: result.code || code,
        map:
          typeof result.map === "string" ? JSON.parse(result.map) : result.map,
      };
      return output;
    },
  };
};

export { terser };
