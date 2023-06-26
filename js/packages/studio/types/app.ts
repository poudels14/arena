import { Widget } from "@arena/widgets";

type App = {
  id: string;

  name: string;

  description?: string;

  config: {
    ui?: {
      thumbnail?: {
        class?: string;
      };
    };
  };

  /**
   * List of all the widgets in the app
   */
  widgets: Record<Widget["id"], Widget>;

  /**
   * List of all the resources available to the app
   */
  resources: Record<string, Resource>;
};

type Resource = {
  id: string;
  type: string;
  name: string;
  description?: string | null;
};

export type { App, Resource };
