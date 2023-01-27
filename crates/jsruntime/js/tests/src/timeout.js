const timeout = setTimeout(() => {
  console.log("long wait...")
}, 5000)

setTimeout(() => {
  console.log("timed out!");
  clearTimeout(timeout);
}, 100);
