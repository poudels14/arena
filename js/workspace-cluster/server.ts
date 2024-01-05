import { Pool } from "@arena/runtime/postgres";

import { router } from "~/api/index";
import { createRepo } from "./api/repo";
import { env } from "./api/env";

const dbpool = new Pool({
  host: env.DATABASE_HOST,
  port: env.DATABASE_PORT,
  database: env.DATABASE_NAME,
  user: env.DATABASE_USER,
  password: env.DATABASE_PASSWORD,
});

const fetch = async (request: any) => {
  try {
    const client = await dbpool.connect();
    const result = await router.route(request, {
      env: process.env,
      context: {
        host: env.HOST,
        env,
        dbpool,
        repo: createRepo(client),
        user: null,
      },
    });

    await client.release();
    return result;
  } catch (e) {
    console.error(e);
    return new Response("Internal Server Error", { status: 500 });
  }
};

export { fetch };
export default { fetch };
