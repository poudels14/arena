const { app, Menu, globalShortcut, dialog, autoUpdater } = require("electron");
import { url as updateUrl } from "./updater";

const setupMenu = () => {
  const menuTemplate = [
    {
      label: app.name,
      submenu: [
        {
          label: "Reset and restart",
          click: async () => {
            dialog
              .showMessageBox({
                type: "info",
                buttons: ["Reset", "Cancel"],
                title: "Application reset",
                message: "Reseting app will clear all your app data",
                detail: "Are you sure you want to reset?",
              })
              .then((returnValue: any) => {
                if (returnValue.response === 0) {
                  const portal =
                    // @ts-expect-error
                    import.meta.env.MODE == "development"
                      ? require(`../../../../crates/target/debug/portal-${__APP_VERSION__}.node`)
                      : require(`./portal-${__APP_VERSION__}.node`);
                  portal.resetData();
                  app.relaunch();
                  app.quit();
                }
              });
          },
        },
        {
          label: "Check for updates...",
          click: async () => {
            const res = await fetch(updateUrl);
            if (res.status == 204) {
              dialog.showMessageBox({
                type: "info",
                buttons: ["Close"],
                title: "App update",
                message: "No update available",
                detail: `You are running the latest version (${__APP_VERSION__}) of Portal.`,
              });
            } else if (res.status == 200) {
              autoUpdater.checkForUpdates();
            }
          },
        },
        { role: "quit" },
      ],
    },
    {
      label: "Edit",
      submenu: [
        { label: "Undo", accelerator: "CmdOrCtrl+Z", selector: "undo:" },
        { label: "Redo", accelerator: "Shift+CmdOrCtrl+Z", selector: "redo:" },
        { type: "separator" },
        { label: "Cut", accelerator: "CmdOrCtrl+X", selector: "cut:" },
        { label: "Copy", accelerator: "CmdOrCtrl+C", selector: "copy:" },
        { label: "Paste", accelerator: "CmdOrCtrl+V", selector: "paste:" },
        {
          label: "Select All",
          accelerator: "CmdOrCtrl+A",
          selector: "selectAll:",
        },
      ],
    },
  ];

  const menu = Menu.buildFromTemplate(menuTemplate);
  Menu.setApplicationMenu(menu);

  // Disable dev tools shortcut
  globalShortcut.register("CommandOrControl+Shift+I", () => {
    console.log("Developer Tools shortcut attempted, but disabled.");
  });
};

export { setupMenu };
