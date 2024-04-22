const {
  app,
  BrowserWindow,
  Menu,
  globalShortcut,
  dialog,
} = require("electron");
const { fork } = require("child_process");
import path from "path";
import * as Sentry from "@sentry/node";
import "./updater";
import { setupMenu } from "./menu";

Sentry.init({
  dsn: "https://b6d70976adbb0932725f7b4817e422cf@o4507128581914624.ingest.us.sentry.io/4507131496366080",
  integrations: [],
  tracesSampleRate: 1.0,
  profilesSampleRate: 0.0,
});

let server: any;
const startServer = async () => {
  const controller = new AbortController();
  const id = setTimeout(() => controller.abort(), 3_000);
  const res = await fetch("http://localhost:42690/_healthy", {
    signal: controller.signal,
  }).catch((e) => {
    return { ok: false };
  });
  clearTimeout(id);

  if (res.ok) {
    dialog
      .showMessageBox({
        type: "info",
        buttons: ["Quit"],
        title: "Multiple instances found",
        message: "Another instance of Portal open",
        detail:
          "Only one instance of Portal is allowed. Please quit all instances and try again.",
      })
      .then((returnValue: any) => {
        app.quit();
      });
  }

  // TODO: use utilityProcess instead; it wasn't working, so used child process
  // for now
  server = fork(path.join(__dirname, "server.js"), [], {
    stdio: "inherit",
  });
};
startServer();

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

app.on("before-quit", () => {
  if (server) {
    server.kill();
  }
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
