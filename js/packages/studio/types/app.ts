import { Widget } from "@arena/widgets";

type App = {
  id: string;

  name: string;

  description?: string;

  /**
   * List of all the widgets in the app
   */
  widgets: Widget[];
};

export type { App };
