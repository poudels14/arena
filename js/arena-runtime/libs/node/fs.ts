const { readFileSync, lstat: statSync } = Arena.fs;

const fs = {
  readFileSync,
  statSync,
};

export { readFileSync, statSync };
export default fs;
