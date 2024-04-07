import { createHandler } from "@solidjs/start/entry";
import { StartServer } from "@solidjs/start/server";
import PortalIcon from "./icons/favicon.ico";
import { env } from "~/env";
// use this here so that env is validated on startup
void env;

export default createHandler(() => (
  <StartServer
    document={({ assets, children, scripts }) => (
      <html lang="en">
        <head>
          <meta charset="utf-8" />
          <meta name="viewport" content="width=device-width, initial-scale=1" />
          <link rel="icon" type="image/x-icon" href={PortalIcon} />
          <link rel="preconnect" href="https://rsms.me/" />
          <link rel="stylesheet" href="https://rsms.me/inter/inter.css" />
          <script
            defer
            data-domain="useportal.ai"
            src="https://plausible.io/js/script.js"
          ></script>
          {assets}
        </head>
        <body class="scroll:w-1 thumb:rounded thumb:bg-slate-700 track:bg-slate-900">
          {children}
        </body>
        {scripts}
      </html>
    )}
  />
));
