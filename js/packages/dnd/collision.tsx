import { Sensor } from "./drag-drop-context";
import { Droppable } from "./droppable";
import { Collision, CollisionOptions } from "./types";
import { distanceBetweenPoints } from "./utils";

const MIN_MOVE_DISTANCE = 5;

const findClosestDroppable = (
  sensor: Sensor,
  droppables: Droppable[],
  options: Required<CollisionOptions>
): Collision | null => {
  // Note(sagar): only check for collision if the move distance is more than
  // a threshold
  if (
    Math.sqrt(Math.pow(sensor.delta.x, 2) + Math.pow(sensor.delta.y, 2)) <
    MIN_MOVE_DISTANCE
  ) {
    return null;
  }
  let minDist = options.distance;
  let closestDroppable: Droppable | null = null;
  droppables.forEach((d) => {
    const rect = d.node.getBoundingClientRect();
    const centerDist = distanceBetweenPoints(
      [rect.x + rect.width / 2, rect.y + rect.height / 2],
      [sensor.current.x, sensor.current.y]
    );
    const topLeftDist = distanceBetweenPoints(
      [rect.x, rect.y],
      [sensor.current.x, sensor.current.y]
    );
    const dist = Math.min(centerDist, topLeftDist);
    if (dist < minDist) {
      minDist = dist;
      closestDroppable = d;
    }
  });

  return closestDroppable
    ? { droppable: closestDroppable, distance: minDist }
    : null;
};

export { findClosestDroppable };
