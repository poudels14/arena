import { z } from "zod";
import { dataSourceSchema } from "./data";
import { templateSchema } from "./template";

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
