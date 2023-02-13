import {
  init,
  WASI,
  MemFS,
  JSVirtualFile,
  WasmerRuntimeError,
} from "./deno/wasi";
import { InitGo } from "./go";

init();

globalThis.__bootstrap.wasi = {
  init,
  WASI,
  MemFS,
  JSVirtualFile,
  WasmerRuntimeError,
  InitGo,
};
