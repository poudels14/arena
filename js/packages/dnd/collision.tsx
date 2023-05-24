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
  const { x: sensorX, y: sensorY } = sensor.current;
  droppables.forEach((d) => {
    const { x, y, width, height } = d.node.getBoundingClientRect();
    // Note(sagar): if the pointer is in between X coords or Y coords of the
    // droppable, make that axis 0 when calculating the distance
    const px0 = sensorX > x && sensorX < x + width ? sensorX : null;
    const py0 = sensorY > y && sensorY < y + height ? sensorY : null;
    const centerDist = distanceBetweenPoints(
      [px0 ?? x + width / 2, py0 ?? y + height / 2],
      [sensorX, sensorY]
    );

    const topLeftDist = distanceBetweenPoints([x, y], [sensorX, sensorY]);
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
