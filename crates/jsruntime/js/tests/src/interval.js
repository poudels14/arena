let count = 0;
const timeout = setInterval(() => {
  console.log("count:", count);
  count++;

  if (count == 3) {
    clearInterval(timeout);
  }
}, 20)
