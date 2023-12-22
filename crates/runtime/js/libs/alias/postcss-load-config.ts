export default async (..._args) => {
  // Note(sagar): even though loading config isn't loaded, need to throw
  // the following error to trick postcss to think config loading is working
  // fine but it's just that config wasn't found
  throw new Error("No PostCSS Config found");
};
