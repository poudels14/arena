import z from "zod";
import { createEnv } from "@t3-oss/env-core";

const env = createEnv({
  server: {
    MODE: z.enum(["development", "production"]),
    POSTHOG_API_KEY: z.string(),
  },
  isServer: true,
  runtimeEnv: process.env,
});

export { env };
