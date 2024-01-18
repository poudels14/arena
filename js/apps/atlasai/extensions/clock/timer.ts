import Component from "./Timer";
import { fromZodError, zod as z } from "@portal/sdk";

const startTimerSchema = z.object({
  duration: z
    .number()
    .optional()
    .describe("Timer duration. Leave this empty if uncertain"),
  unit: z.enum(["minute", "second"]).optional().describe("Unit of time"),
});

export const manifest = {
  name: "start_timer",
  description: "Start a timer for a given duration",
  schema: startTimerSchema,
  async start(options: {
    input: any;
    prompt: (message: string) => Promise<void>;
    setState: (state: any) => Promise<void>;
  }) {
    const { input } = options;

    const parsed = startTimerSchema.safeParse(options.input);
    if (!parsed.success) {
      throw new Error(fromZodError(parsed.error!).toString());
    }
    const durationInSeconds =
      input.unit == "second"
        ? input.duration
        : input.unit == "minute"
        ? input.duration * 60
        : 0;
    const startedAt = new Date().getTime();
    await options.setState({
      duration: durationInSeconds * 1000,
      startedAt,
    });
  },
};

export default Component;
