import pkg from "./package.json";
import { defineConfig } from "@portal/deploy/bundle/solid-start";
// import { defineConfig } from "@solidjs/start/config";

const latestDesktopVersions = pkg["portal-config"]["latest-desktop-versions"];
export default defineConfig({
  upload: true,
  // assetsInclude: ["**/*.ico"],
  define: {
    _MAC_APP_VERSION: JSON.stringify(latestDesktopVersions.mac),
    _LINUX_APP_VERSION: JSON.stringify(latestDesktopVersions.linux),
  },
});
