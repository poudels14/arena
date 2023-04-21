export const Logger = {
  child(...a: any) {
    return {
      debug(...args: any) {
        console.debug(args);
      },
      error(...args: any) {
        console.error(args);
      },
      warn(...args: any) {
        console.warn(args);
      },
    };
  },
};
