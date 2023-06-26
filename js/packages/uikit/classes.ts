import { clsx } from "clsx";

/**
 * Filters Tailwind classes related to text like "text-xs", "text-green-100",
 * etc
 */
const filterTextClasses = (classList: Record<string, boolean>) => {
  return clsx(classList)
    .split(" ")
    .filter((c) => c.startsWith("text"))
    .join(" ");
};

export { clsx, filterTextClasses };
