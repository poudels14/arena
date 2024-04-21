const { app, BrowserWindow, Menu, globalShortcut } = require("electron");
import path from "path";

// TODO: use utilityProcess instead; it wasn't working, so used child process
// for now
const { fork } = require("child_process");
const server = fork(path.join(__dirname, "server.js"), [], {
  stdio: "inherit",
});

const createWindow = () => {
  const win = new BrowserWindow({
    width: 1000,
    height: 700,
  });
  win.setMinimumSize(100, 80);

  // and load the index.html of the app.
  if (MAIN_WINDOW_VITE_DEV_SERVER_URL) {
    win.loadURL(MAIN_WINDOW_VITE_DEV_SERVER_URL);
  } else {
    win.loadFile(
      path.join(__dirname, `../renderer/${MAIN_WINDOW_VITE_NAME}/index.html`)
    );
  }
  // @ts-expect-error
  if (import.meta.env.MODE == "development") {
    win.webContents.openDevTools();
  }
};

const setupMenu = () => {
  const menuTemplate = [
    {
      label: app.name,
      submenu: [{ role: "quit" }],
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

app.on("before-quit", () => {
  server.kill();
});

app.whenReady().then(() => {
  setupMenu();
  createWindow();
});

app.on("window-all-closed", () => {
  if (process.platform !== "darwin") {
    app.quit();
  }
});

app.on("activate", () => {
  // On OS X it's common to re-create a window in the app when the
  // dock icon is clicked and there are no other windows open.
  if (BrowserWindow.getAllWindows().length === 0) {
    createWindow();
  }
});
