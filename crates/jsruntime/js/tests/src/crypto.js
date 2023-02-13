if (!crypto) {
  throw new Error("crypto is undefined");
}

console.log(crypto.getRandomValues);
