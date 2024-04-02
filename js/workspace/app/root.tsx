import { Html, Head, Body, Link, Meta } from "@portal/solidjs";
import { Workspace } from "./Workspace";
import "./style.css";

const Root = () => {
  return (
    <Html lang="en">
      <Head>
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <Link rel="preconnect" href="https://rsms.me/" />
        <Link rel="stylesheet" href="https://rsms.me/inter/inter.css" />
        <Link
          rel="stylesheet"
          type="text/css"
          href={process.env.PORTAL_STYLE_CSS || "./static/style.css"}
        />
      </Head>
      <Body class="antialiased">
        <Workspace />
      </Body>
    </Html>
  );
};

export default Root;
