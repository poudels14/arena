import path from "path";

const loadConfig = async () => {
  return await Arena.fs
    .readAsJson(path.join(process.cwd(), "./workspace.config.toml"))
    .then((config) => {
      console.log("Config loaded from ./workspace.config.toml");
      return JSON.parse(config);
    })
    .catch((_) => {
      console.log("Error loading workspace.config.toml. Using default configs");
      return {};
    });
};

export { loadConfig };
