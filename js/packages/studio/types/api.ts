import { Widget } from "@arena/widgets";

type EntityUpdates<T> = {
  /**
   * This list of entities that were created
   */
  created?: T[];
  /**
   * This list of entities that were updated
   */
  updated?: T[];
  /**
   * This list of entities that were deleted
   */
  deleted?: T[];
};

type MutationResponse = {
  /**
   * List of widgets affected by the mutation
   */
  widgets?: EntityUpdates<Widget>;

  resources?: EntityUpdates<any>;
};

export type { EntityUpdates, MutationResponse };
