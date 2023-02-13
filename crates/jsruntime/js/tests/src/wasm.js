console.log(Arena.wasi);

const { ops } = Deno.core;
const { init, WASI, Go } = Arena.wasi;
const go = new Go();

/************************* RUNNING Go wasm ************************/

console.log("Running Golang wasm");
console.log("-------------------------------------------------------");

const goWasmFile = "./golang.wasm";
const goWasmContent = ops.op_read_file_sync(goWasmFile);

await WebAssembly.instantiate(goWasmContent, {
  ...go.importObject,
}).then(async (m) => {
   const { instance } = m;
  go.run(instance);
});

console.log("********************************************************");
console.log();
console.log();
/************************* RUNNING non-Go wasm ************************/
console.log("Running non-Go wasm");
console.log("-------------------------------------------------------");

const nonGoWasmFile = "./non-golang.wasm";
const nonGoWasmContent = ops.op_read_file_sync(nonGoWasmFile);

const nonGoWasmModule = await WebAssembly.compile(nonGoWasmContent);

await init();
let wasi = new WASI({
  env: {},
  args: [],
});
const nonGoWasmInstance = await wasi.instantiate(nonGoWasmModule, {});

// Run the start function
let exitCode = wasi.start();
let stdout = wasi.getStdoutString();

// This should print "hello world (exit code: 0)"
console.log(`${stdout}(exit code: ${exitCode})`);

console.log("********************************************************");
