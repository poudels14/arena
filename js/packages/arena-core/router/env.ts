// Note(sp): this is to prevent env replacement during build
const isDev = () => {
  const env = process.env;
  return env["NODE_ENV"] == "development";
};

export { isDev };
