import { Show, lazy, Switch, Match } from "solid-js";
import { Body, Head, Html, Link, useServerContext } from "@arena/core/solid";

const Homepage = lazy(() => import("./routes/homepage/index.tsx"));
const Waitlisted = lazy(() => import("./routes/waitlist.tsx"));
const ProtectedRoutes = lazy(() => import("./routes/index.tsx"));

export default function Root() {
  const { user } = useServerContext<any>();
  const getUser = () =>
    user ||
    JSON.parse(
      document.getElementById("data/loggedInUser")?.textContent || "null"
    );

  return (
    <Html lang="en">
      <Head>
        <Link rel="preconnect" href="https://rsms.me/" />
        <Link rel="stylesheet" href="https://rsms.me/inter/inter.css" />
        <style>
          {`:root { font-family: 'Inter', sans-serif; }
            @supports (font-variation-settings: normal) {
              :root { font-family: 'Inter var', sans-serif; }
            }
          `}
        </style>
        <Show when={user}>
          <script type="application/json" id="data/loggedInUser">
            {JSON.stringify(user)}
          </script>
        </Show>
      </Head>
      <Body class="antialiased">
        <Switch>
          <Match when={!getUser().id}>
            <Homepage />
          </Match>
          <Match when={getUser().config?.waitlisted}>
            <Waitlisted />
          </Match>
          <Match when={true}>
            <ProtectedRoutes user={getUser()} />
          </Match>
        </Switch>
      </Body>
    </Html>
  );
}
