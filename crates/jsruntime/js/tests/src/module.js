console.log(import.meta);

// new properties can be added to import.meta
import.meta.name = "TEST NAME";

console.log(import.meta);

console.log(import.meta.resolve("./interval"));
