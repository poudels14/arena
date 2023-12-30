const isatty = () => false;

const tty = { isatty };

Arena.__nodeInternal = {
  ...(Arena.__nodeInternal || {}),
  tty,
};

export default tty;
export { isatty };
