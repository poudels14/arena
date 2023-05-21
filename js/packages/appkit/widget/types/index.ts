import { z } from "zod";
import { dataSourceSchema } from "./data";

export const templateSchema = z.object({
  /**
   * Id of the template
   */
  id: z.string(),

  /**
   * Name of the template
   */
  name: z.string(),

  /**
   * Url to load the component from
   */
  url: z.string(),
});

export const widgetConfigSchema = z.object({
  data: z.record(dataSourceSchema),
  class: z.string().optional(),
});

export const widgetSchema = z.object({
  /**
   * Id of the widget
   */
  id: z.string(),
  name: z.string(),
  slug: z.string(),
  description: z.string().optional(),

  /**
   * parentId is null for root widgets
   */
  parentId: z.string().nullable(),
  template: templateSchema,
  config: widgetConfigSchema,
});

export type WidgetConfig = z.infer<typeof widgetConfigSchema>;
export type Widget = z.infer<typeof widgetSchema>;
export type { DataSource } from "./data";
export type { Template } from "./template";
