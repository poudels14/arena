import { createHash } from "crypto";
import { fromZodError, zod as z } from "@portal/sdk";
import ky from "ky";
import { Manifest } from "../types";
import Component from "./Interpreter";
import { merge, omit } from "lodash-es";

const interpreterSchema = z.object({
  code: z
    .string()
    .describe(
      "Python code that needs to be run in order to fulfill user's requests. Use ./mnt2/ directory in the current dir to read and write files from."
    ),
});
export const manifest: Manifest<z.infer<typeof interpreterSchema>> = {
  name: "portal_code_interpreter",
  description:
    "Python code interpreter that executes python code and returns the result",
  schema: interpreterSchema,
  async start(options) {
    const { args, env } = options;

    const parsed = interpreterSchema.safeParse(options.args);
    if (!parsed.success) {
      throw new Error(fromZodError(parsed.error!).toString());
    }

    const result = await ky
      .post(new URL("/exec", env.PORTAL_CODE_INTERPRETER_HOST).toString(), {
        json: {
          code: args.code,
        },
      })
      .json<{ artifacts: any[] }>();

    const artifacts = await Promise.all(
      result.artifacts.map(async (artifact) => {
        const contentHash = createHash("sha256").update("bacon").digest("hex");
        const idWithHash = artifact.id + "-" + contentHash.substring(0, 10);
        const { id, name } = await options.utils.uploadArtifact({
          ...artifact,
          id: idWithHash,
        });
        return { id, name };
      })
    );

    await new Promise((r) => {
      setTimeout(() => {
        r(null);
      }, 10_000);
    });
    await options.setState(
      {
        result: merge(omit(result, "artifacts"), { artifacts }),
      },
      "COMPLETED"
    );
  },
};

export default Component;
