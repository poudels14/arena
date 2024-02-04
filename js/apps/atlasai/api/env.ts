import { createEnv } from "@t3-oss/env-core";
import z from "zod";

const env = createEnv({
  server: {
    MODE: z.enum(["development", "production"]).default("development"),
    PORTAL_WORKSPACE_HOST: z.string(),
    PORTAL_DATABASE_HOST: z.string(),
    PORTAL_DATABASE_PORT: z.string().transform((val) => parseInt(val)),
    PORTAL_DATABASE_NAME: z.string(),
    PORTAL_DATABASE_USER: z.string(),
    PORTAL_DATABASE_PASSWORD: z.string(),
    OPENAI_API_KEY: z.string(),
  },
  runtimeEnv: process.env,
});

type Env = typeof env;

export { env };
export type { Env };
