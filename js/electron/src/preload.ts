const { contextBridge, ipcRenderer } = require("electron/renderer");

contextBridge.exposeInMainWorld("electronAPI", {
  reportServerStartError: (retry: any) =>
    ipcRenderer.send("error:report-server-start-error", retry),
});
