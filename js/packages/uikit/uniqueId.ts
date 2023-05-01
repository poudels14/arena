import { customAlphabet } from "nanoid";

const alphabet = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghjkmnpqrstuvwxyz";
const uniqueId = customAlphabet(alphabet, 22);

export { uniqueId };
