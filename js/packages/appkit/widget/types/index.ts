import { DataSource } from "./data";
import { Template } from "./template";

type Template = {
  /**
   * Id of the template
   */
  id: string;

  /**
   * Name of the template
   */
  name: string;

  /**
   * Url to load the component from
   */
  url: string;
};

export type WidgetConfig = {
  data: Record<string, DataSource<any>>;
  classList?: string[];
};

export type Widget = {
  /**
   * Id of the widget
   */
  id: string;
  name: string;
  slug: string;
  description?: string;
  parentId: string | null;
  template: Template;
  config: WidgetConfig;
};

export type { DataSources } from "./data";
