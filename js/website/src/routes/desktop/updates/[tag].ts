import semver from "semver";
import { APIEvent } from "@solidjs/start/server";

const MAC_VERSIONS = [
  "0.1.9",
  "0.1.10",
  "0.1.11",
  "0.1.12",
  "0.1.13",
  "0.1.14",
];
export async function GET({ params }: APIEvent) {
  const regex = [...params.tag.matchAll(/(\w+)-(\w+)-([0-9\.]+)/gm)];
  const os = regex[0][1];
  const arch = regex[0][2];
  const currentVersion = regex[0][3];
  try {
    if (os == "darwin") {
      const newVersion = MAC_VERSIONS.findLast((version) =>
        semver.gt(version, currentVersion)
      );
      console.log(
        `new version for mac [current=${currentVersion}, arch=${arch}]: `,
        newVersion
      );
      if (newVersion) {
        return {
          version: newVersion,
          pub_date: new Date().toISOString(),
          url: `https://github.com/poudels14/portal-release/releases/download/${newVersion}/Portal-${newVersion}-${arch}-mac.zip`,
        };
      }
    }
  } catch {}

  return new Response(null, {
    status: 204,
  });

  // return {
  //   version: newVersion,
  //   pub_date: new Date().toISOString(),
  //   url: `https://github.com/poudels14/portal-release/releases/download/${newVersion}/Portal_${newVersion}_${arch}.dmg`,
  //   signature: "Content of the relevant .sig file",
  //   notes: "These are some release notes",
  // };
}
