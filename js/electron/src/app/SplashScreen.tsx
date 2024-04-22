import {
  createSignal,
  createResource,
  createEffect,
  Switch,
  Match,
} from "solid-js";
import * as Sentry from "@sentry/browser";
import Logo from "../../../../logos/portal-blue.png";

const HOST = "http://localhost:42690/";
const SplashScreen = (props: any) => {
  const [serverReady, setServerReady] = createSignal(false);
  const [setupError, setSetupError] = createSignal("");

  const RETRY = 50;
  const pollServer = async (retry: number) => {
    const controller = new AbortController();
    const id = setTimeout(() => controller.abort(), 2_500);
    const res = await fetch(new URL("/_healthy", HOST), {
      signal: controller.signal,
    }).catch((e) => {
      return { ok: false };
    });
    clearTimeout(id);
    if (res.ok) {
      setServerReady(true);
    } else {
      if (retry == 0) {
        Sentry.captureException(
          new Error(`Error fetching /_healthy [retry count = ${RETRY}]`)
        );
        setSetupError("Something went wrong. Please restart the app");
        return;
      }
      setTimeout(() => {
        pollServer(retry - 1);
      }, 1000);
    }
  };

  pollServer(RETRY);
  return (
    <div class="w-screen h-screen flex justify-center items-center bg-slate-100">
      <div class="space-y-10">
        <div class="text-center space-y-3">
          <div class="flex justify-center">
            <img src={Logo} class="w-24" />
          </div>
          <div class="text-5xl font-bold text-gray-700">Portal</div>
        </div>
        <div class="text-gray-600">
          <Switch>
            <Match when={setupError()}>
              <div>{setupError()}</div>
            </Match>
            <Match when={!serverReady()}>
              <div class="text-center">Loading...</div>
            </Match>
            <Match when={serverReady()}>
              <Setup
                onReady={() => {
                  window.location.href = new URL("/", HOST).href;
                }}
              />
            </Match>
          </Switch>
        </div>
      </div>
    </div>
  );
};

const Setup = (props: { onReady: () => void }) => {
  const [workspaces] = createResource<any[]>(() =>
    fetch(new URL("/api/workspaces", HOST)).then((r) => r.json())
  );

  const [defaultWorkspace] = createResource<any, any>(
    () => workspaces()?.[0],
    (workspace) => {
      return fetch(new URL(`/api/workspaces/${workspace.id}`, HOST)).then((r) =>
        r.json()
      );
    }
  );

  const [atlasAppReady] = createResource(
    () => {
      const workspace = defaultWorkspace();
      if (workspace) {
        const atlas = workspace.apps.find((app: any) => app.slug == "atlas_ai");
        if (atlas) {
          return atlas;
        }
      }
    },
    (atlas) => {
      return fetch(new URL(`/w/apps/${atlas.id}/_admin/healthy`, HOST)).then(
        (res) => {
          return res.ok;
        }
      );
    }
  );

  createEffect(() => {
    if (atlasAppReady()) {
      props.onReady();
    } else {
      // trigger ready after some timeout to prevent app from getting
      // stuck on splash screen
      setTimeout(() => {
        props.onReady();
      }, 15_000);
    }
  });
  return <div class="text-center">Setting up your workspace...</div>;
};

export default SplashScreen;
