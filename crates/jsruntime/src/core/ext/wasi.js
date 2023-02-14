// "use strict";
// ((global) => {
//   const {
//     init,
//     WASI,
//     MemFS,
//     JSVirtualFile,
//     WasmerRuntimeError,
//     InitGo,
//   } = global.__bootstrap.wasi;

//   const Go = InitGo(global);
//   global.Arena.wasi = {
//     init,
//     WASI,
//     MemFS,
//     JSVirtualFile,
//     WasmerRuntimeError,
//     Go
//   }
// })(globalThis);