import { Widget } from "@arena/widgets";

type App = {
  id: string;

  name: string;

  description?: string;

  /**
   * List of all the widgets in the app
   */
  widgets: Record<Widget["id"], Widget>;

  /**
   * List of all the resources available to the app
   */
  resources: Record<
    string,
    {
      id: string;
      name: string;
      description: string | null;
      type: string;
    }
  >;
};

export type { App };
