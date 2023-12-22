import {
  init,
  WASI,
  MemFS,
  JSVirtualFile,
  WasmerRuntimeError,
} from "./deno/wasi";
import { InitGo } from "./go";

init();

const Go = InitGo(globalThis);
globalThis.Arena.wasi = {
  init,
  WASI,
  MemFS,
  JSVirtualFile,
  WasmerRuntimeError,
  Go,
};
