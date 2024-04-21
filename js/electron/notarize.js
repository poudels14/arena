const { notarize } = require("@electron/notarize");
const pkg = require("./package.json");

exports.default = async function notarizing(context) {
  const { electronPlatformName, appOutDir } = context;
  if (electronPlatformName !== "darwin") {
    return;
  }

  const appName = context.packager.appInfo.productFilename;
  return await notarize({
    appBundleId: pkg.build.appId,
    appPath: `${appOutDir}/${appName}.app`,
    appleId: process.env.APPLE_ID,
    appleIdPassword: process.env.APPLE_PASSWORD,
    teamId: process.env.APPLE_TEAM_ID,
  });
};
