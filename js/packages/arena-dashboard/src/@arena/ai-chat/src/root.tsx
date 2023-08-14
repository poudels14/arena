import App from "./app";
import { Body, Head, Html, Link } from "@arena/core/solid";

export default function Root() {
  return (
    <Html lang="en">
      <Head>
        {/* TODO(sagar): remove these in prod build */}
        <Link rel="preconnect" href="https://rsms.me/" />
        <Link rel="stylesheet" href="https://rsms.me/inter/inter.css" />
        <style>
          {`:root { font-family: 'Inter', sans-serif; }
            @supports (font-variation-settings: normal) {
              :root { font-family: 'Inter var', sans-serif; }
            }
          `}
        </style>
      </Head>
      <Body class="antialiased">
        <App />
      </Body>
    </Html>
  );
}
