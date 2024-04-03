// import { Html, Head, Body, Link } from "@portal/solidjs";
// import { Route, Router } from "@portal/solid-router";
// import { Setup } from "./setup";
// import { Show } from "solid-js";

// const Root = () => {
//   const workspace = null;
//   return (
//     <Html lang="en">
//       <Head>
//         {/* <Link rel="preconnect" href="https://rsms.me/" />
//         <Link rel="stylesheet" href="https://rsms.me/inter/inter.css" /> */}
//         <Link rel="stylesheet" type="text/css" href="/app/style.css" />
//         <style>
//           {`:root { font-family: 'Inter', sans-serif; }
//             @supports (font-variation-settings: normal) {
//               :root { font-family: 'Inter var', sans-serif; }
//             }
//           `}
//         </style>
//       </Head>
//       <Body class="antialiased">
//         {/* <Router> */}
//         {/* <Route path="/setup">
//             <Setup />
//           </Route> */}
//         {/* </Router> */}
//         <Show when={!workspace}>
//           <Setup />
//         </Show>
//       </Body>
//     </Html>
//   );
// };

import { Html, Head, Body, Link } from "@portal/solidjs";
import { Workspace } from "@portal/workspace/app/Workspace";
import "./style.css";

const Root = () => {
  return (
    <Html lang="en">
      <Head>
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <Link
          rel="stylesheet"
          type="text/css"
          href={process.env.PORTAL_STYLE_CSS || "./static/style.css"}
        />
        <link
          rel="stylesheet"
          type="text/css"
          href={`${process.env.PORTAL_ASSETS_BASE}/fonts/inter/inter.css`}
        />
      </Head>
      <Body class="antialiased select-none">
        <Workspace />
      </Body>
    </Html>
  );
};

export default Root;
