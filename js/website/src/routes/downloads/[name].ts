import { APIEvent } from "@solidjs/start/server";
import { uniqueId } from "@portal/sdk/utils/uniqueId";
import { redirect } from "@solidjs/router";
import ky from "ky";
import { env } from "~/env";

export async function GET({ params }: APIEvent) {
  const regex = [...params.name.matchAll(/Portal_([0-9.]+)_(\w+).(\w+)/gm)];
  console.log("regex =", [...regex]);
  const version = regex[0][1];
  const arch = regex[0][2];
  const fileType = regex[0][3];
  if (!version || !arch || !fileType) {
    return new Response("Not found", {
      status: 404,
    });
  }

  if (env.MODE == "production") {
    await ky
      .post("https://app.posthog.com/capture/", {
        json: {
          api_key: env.POSTHOG_API_KEY,
          event: "downloads",
          distinct_id: uniqueId(),
          properties: {
            arch,
            version,
            fileType,
          },
          timestamp: new Date().toISOString(),
        },
      })
      .json();
  }

  return redirect(
    `https://github.com/poudels14/portal-release/releases/download/${version}/Portal_${version}_${arch}.${fileType}`
  );
}
