import { Show, For, createResource } from "solid-js";
import { Title } from "@arena/core/solid";
import { A } from "@solidjs/router";
import { useDashboardContext } from "~/context";

const App = (props: {
  id: string;
  name: string;
  description: string;
  access: string[];
}) => {
  return (
    <A
      href={"/apps/" + props.id}
      class="w-80 h-40 block relative group bg-brand-2 rounded-lg bg-gradient-to-r from-cyan-300 to-blue-300 cursor-pointer"
    >
      <div class="absolute bottom-0 px-4 py-2">
        <div class="font-medium text-brand-11 group-hover:text-brand-12">
          {props.name}
        </div>
      </div>
    </A>
  );
};

const Apps = () => {
  const { client } = useDashboardContext();

  const [apps] = createResource(() => {
    return client.apps.listApps.query();
  });

  return (
    <Show when={apps()}>
      <Title>Apps</Title>
      <div class="">
        <div class="mt-40 px-16">
          <For each={apps()} fallback="No apps">
            {(app) => {
              return <App {...app} />;
            }}
          </For>
        </div>
      </div>
    </Show>
  );
};

export default Apps;
