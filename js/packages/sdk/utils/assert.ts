/**
 * Throw error when a given value is null or undefined.
 *
 * For example, `assertNotNil(x, "Message")` throws error
 * `Message can't be null` when x is null.
 */
const notNil = (value: any, name: string) => {
  let isNull = value == null;
  let isUndefined = value == undefined;
  if (isNull || isUndefined) {
    throw new Error(`${name} can't be ${isNull ? "null" : "undefined"}`);
  }
};

const assert = { notNil };
export { assert };
