import { customAlphabet } from "nanoid";

const alphabet = "123456789ABCDEFGHJKMNPQRSTUVWXYZabcdefghjkmnpqrstuvwxyz";
const uniqueId = customAlphabet(alphabet, 13);

export { uniqueId };
