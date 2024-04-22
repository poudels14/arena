const { app, autoUpdater, dialog } = require("electron");
import os from "os";

const setupUpdater = () => {
  const url = new URL(
    `/desktop/updates/${os.platform()}/${os.arch()}/${app.getVersion()}`,
    __PORTAL_CLOUD_HOST__
  ).href;
  autoUpdater.setFeedURL({ url });

  autoUpdater.on("update-downloaded", (e: any) => {
    dialog
      .showMessageBox({
        type: "info",
        buttons: ["Restart", "Later"],
        title: "Application Update",
        message: "New version available",
        detail:
          "A new version has been downloaded. Restart the application to apply the updates.",
      })
      .then((returnValue: any) => {
        if (returnValue.response === 0) {
          autoUpdater.quitAndInstall();
        }
      });
  });

  autoUpdater.on("error", (message: any) => {
    console.error("Update error:", message);
  });

  if (app.isPackaged) {
    // check for update every 10 minutes
    setInterval(() => {
      autoUpdater.checkForUpdates();
    }, 60_000 * 10);
  }
};

setupUpdater();
