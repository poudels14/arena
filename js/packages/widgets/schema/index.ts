import { z } from "zod";
import { widgetConfigSchema, widgetSchema } from "./widget";

export const widget = widgetSchema;
export type WidgetConfig = z.infer<typeof widgetConfigSchema>;
export type Widget = z.infer<typeof widgetSchema>;

export { widgetConfigSchema } from "./widget";
export * from "./data";
export type { Template } from "./template";
