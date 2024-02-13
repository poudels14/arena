import { Manifest } from "../types";
import Component from "./Timer";
import { fromZodError, zod as z } from "@portal/sdk";

const startTimerSchema = z.object({
  duration: z
    .number()
    .optional()
    .describe("Timer duration. Leave this empty if uncertain"),
  unit: z.enum(["minute", "second"]).optional().describe("Unit of time"),
});

export const manifest: Manifest<z.infer<typeof startTimerSchema>> = {
  name: "start_timer",
  description: "Start a timer for a given duration",
  schema: startTimerSchema,
  async start(options) {
    const { args } = options;

    const parsed = startTimerSchema.safeParse(options.args);
    if (!parsed.success) {
      throw new Error(fromZodError(parsed.error!).toString());
    }
    const durationInSeconds =
      args.unit == "second"
        ? args.duration!
        : args.unit == "minute"
        ? args.duration! * 60
        : 0;
    const startedAt = new Date().getTime();
    await options.setState({
      duration: durationInSeconds * 1000,
      startedAt,
    });
  },
};

export default Component;
