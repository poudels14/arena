import semver from "semver";
import { p } from "./procedure";

const MAC_VERSIONS = ["0.1.9", "0.1.10", "0.1.11"];

const hasDesktopAppUpdate = p.query(async ({ ctx, req, params, errors }) => {
  const { os, arch, currentVersion } = params;
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
          url: `https://github.com/poudels14/portal-release/releases/download/${newVersion}/Portal_${newVersion}_${arch}_darwin.zip`,
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
});

export { hasDesktopAppUpdate };
