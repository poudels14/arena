import { format, deprecate, debuglog, inspect, promisify } from "util";

const util = { format, deprecate, debuglog, inspect, promisify };

Arena.__nodeInternal = {
  ...(Arena.__nodeInternal || {}),
  util,
};

export default util;
export { format, deprecate, debuglog, inspect, promisify };
