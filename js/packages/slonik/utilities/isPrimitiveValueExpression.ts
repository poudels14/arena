export const isPrimitiveValueExpression = (
  maybe: unknown
): maybe is boolean | number | string | null => {
  return (
    typeof maybe === "string" ||
    typeof maybe === "number" ||
    typeof maybe === "boolean" ||
    // TODO(sagar): add this so that object is sent to Rust as is
    typeof maybe === "object" ||
    maybe === null
  );
};
