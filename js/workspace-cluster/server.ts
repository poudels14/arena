import { Pool } from "@arena/runtime/postgres";
import { Client as S3Client } from "@arena/cloud/s3";

import { router } from "~/api/index";
import { createRepo } from "./api/repo";
import { env } from "./api/env";

const dbpool = new Pool({
  host: env.DATABASE_HOST,
  port: env.DATABASE_PORT,
  database: env.DATABASE_NAME,
  user: env.DATABASE_USER,
  password: env.DATABASE_PASSWORD || "",
});

const fetch = async (request: any) => {
  const repo = await createRepo({ pool: dbpool });
  try {
    const result = await router.route(request, {
      env: process.env,
      context: {
        host: env.HOST,
        env,
        dbpool,
        repo,
        user: null,
        s3Client: new S3Client({
          region: {
            Custom: {
              region: "N/A",
              endpoint: env.S3_ENDPOINT || "http://localhost:8001",
            },
          },
          credentials: {
            access_key: env.S3_ACCESS_KEY,
            secret_key: env.S3_SECRET_KEY,
          },
          withPathStyle: true,
        }),
      },
    });
    return result;
  } catch (e) {
    console.error(e);
    return new Response("Internal Server Error", { status: 500 });
  } finally {
    await repo.release();
  }
};

export { fetch };
export default { fetch };
