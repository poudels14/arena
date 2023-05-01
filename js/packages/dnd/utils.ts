import { Point } from "./types";

const distanceBetweenPoints = (first: Point, second: Point) => {
  return Math.sqrt(
    Math.pow(first[0] - second[0], 2) + Math.pow(first[1] - second[1], 2)
  );
};

export { distanceBetweenPoints };
