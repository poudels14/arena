import { HFTokenizer } from "./tokenizer";

const tokenizer = await HFTokenizer.init(
  "sentence-transformers/all-MiniLM-L6-v2"
);
const tokens = await tokenizer.tokenize("Hello there!");
console.log(tokens);
