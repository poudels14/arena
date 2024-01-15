import { createEnv } from "@t3-oss/env-core";
import z from "zod";

const env = createEnv({
  server: {
    MODE: z.enum(["development", "production"]).default("development"),
    DATABASE_HOST: z.string(),
    DATABASE_PORT: z.string().transform((val) => parseInt(val)),
    DATABASE_NAME: z.string(),
    DATABASE_USER: z.string(),
    DATABASE_PASSWORD: z.string(),
  },
  runtimeEnv: process.env,
});

type Env = typeof env;

export { env };
export type { Env };
