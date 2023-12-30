import { format, deprecate, debuglog, inspect } from "util";

const util = { format, deprecate, debuglog, inspect };

Arena.__nodeInternal = {
  ...(Arena.__nodeInternal || {}),
  util,
};

export default util;
export { format, deprecate, debuglog, inspect };
