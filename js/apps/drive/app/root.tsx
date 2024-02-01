import { Html, Head, Body, Link } from "@portal/solidjs";
import { FileExplorer } from "./FileExplorer";
import { QueryContextProvider } from "@portal/solid-query";
import { Router } from "@portal/solid-router";

const Root = () => {
  return (
    <Html lang="en">
      <Head>
        <Link rel="preconnect" href="https://rsms.me/" />
        <Link rel="stylesheet" href="https://rsms.me/inter/inter.css" />
        <Link rel="stylesheet" type="text/css" href="/app/style.css" />
        <style>
          {`:root { font-family: 'Inter', sans-serif; }
            @supports (font-variation-settings: normal) {
              :root { font-family: 'Inter var', sans-serif; }
            }
          `}
        </style>
      </Head>
      <Body class="antialiased">
        <QueryContextProvider urlPrefix="/">
          <Router>
            <FileExplorer />
          </Router>
        </QueryContextProvider>
      </Body>
    </Html>
  );
};

export default Root;
