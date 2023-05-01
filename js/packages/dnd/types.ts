import { Droppable } from "./droppable";

export type Id = string | number;

export type Point = [x: number, y: number];

export type Coordinates = {
  x: number;
  y: number;
};

export type Sensor = {
  id: Id;
  origin: Coordinates;
  current: Coordinates;
  get delta(): Coordinates;
};

export type Collision = {
  distance: number;
  droppable: Droppable;
};
