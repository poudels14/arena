import { createEnv } from "@t3-oss/env-core";
import z from "zod";

const env = createEnv({
  server: {
    MODE: z.enum(["development", "production"]).default("development"),
    HOST: z.string().url(),
    DATABASE_HOST: z.string(),
    DATABASE_PORT: z.string().transform((val) => parseInt(val)),
    DATABASE_NAME: z.string(),
    DATABASE_USER: z.string(),
    DATABASE_PASSWORD: z.string().optional(),
    S3_ENDPOINT: z.string(),
    S3_ACCESS_KEY: z.string(),
    S3_ACCESS_SECRET: z.string(),

    // Api key used to authorize access to server bundle
    REGISTRY_API_KEY: z.string(),
    JWT_SIGNING_SECRET: z.string(),
    // for exmple: signin@emails.tryarena.io
    LOGIN_EMAIL_SENDER: z.string().email(),
    RESEND_API_KEY: z.string(),
  },
  runtimeEnv: process.env,
});

type Env = typeof env;

export { env };
export type { Env };
