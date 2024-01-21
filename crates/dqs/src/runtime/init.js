Deno.core.setMacrotaskCallback(globalThis.__bootstrap.handleTimerMacrotask);

globalThis.__setupRuntime();

delete globalThis["__setupRuntime"];

// TODO(sagar): remove this and figure out a way to write to log file instead
globalThis.console = new globalThis.__bootstrap.Console(Deno.core.print);

// Remove bootstrapping data from the global scope
delete globalThis.__bootstrap;
delete globalThis.bootstrap;

// Disable printing function toString to "hide" code of functions
// that use ops calls
Function.prototype.toString = function () {
  return `\x1b[36m[${this[Symbol.toStringTag]}: ${this.name}]\x1b[0m`;
};

// Delete reference to global Arena that has lots of runtime features
// and only provide access to select few features/configs
delete globalThis["Deno"];
// delete globalThis["Arena"];
// TODO: delete global Arena