import { Sensor } from "../drag-drop-context";
import { Droppable } from "../droppable";
import { Collision, Point } from "../types";
import { distanceBetweenPoints } from "../utils";

const MIN_MOVE_DISTANCE = 5;

const findDroppableWithClosestCenter = (
  sensor: Sensor,
  droppables: Droppable[]
): Collision | null => {
  // Note(sagar): only check for collision if the move distance is more than
  // a threshold
  if (
    Math.sqrt(Math.pow(sensor.delta.x, 2) + Math.pow(sensor.delta.y, 2)) <
    MIN_MOVE_DISTANCE
  ) {
    return null;
  }
  let minDist = Infinity;
  let closestDroppable: Droppable;
  droppables.forEach((d) => {
    const rect = d.node.getBoundingClientRect();
    const dist = distanceBetweenPoints(
      [rect.x, rect.y],
      [sensor.current.x, sensor.current.y]
    );
    if (dist < minDist) {
      minDist = dist;
      closestDroppable = d;
    }
  });

  return { droppable: closestDroppable!, distance: minDist };
};

export { findDroppableWithClosestCenter };
