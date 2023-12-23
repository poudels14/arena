type FileLoaderOptions = {
  env?: Record<string, string>;
  resolve?: ResolverConfig;
};

export const createFileRouter: (
  options: FileLoaderOptions
) => (req: Request) => Promise<Response | undefined>;
