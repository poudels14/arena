import { APIEvent } from "@solidjs/start/server";
import { uniqueId } from "@portal/sdk/utils/uniqueId";
import { redirect } from "@solidjs/router";
import ky from "ky";
import { env } from "~/env";

export async function GET({ params }: APIEvent) {
  let filename = "portal_0.1.0_aarch64.dmg";
  if (params.os == "mac") {
    filename = "portal_0.1.0_aarch64.dmg";
  } else if (params.os == "linux-appimage") {
    filename = "portal_0.1.0_amd64.AppImage";
  } else if (params.os == "linux-deb") {
    filename = "portal_0.1.0_amd64.deb";
  } else {
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
            os: params.os,
          },
          timestamp: new Date().toISOString(),
        },
      })
      .json();
  }

  return redirect(
    `https://github.com/poudels14/portal-release/releases/download/0.1.1/${filename}`
  );
}
