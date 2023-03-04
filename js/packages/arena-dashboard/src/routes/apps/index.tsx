import { Title } from "@arena/core/solid";
import { For } from "solid-js";

const App = (props: {
  id: string;
  title: string;
  description: string;
  access: string[];
}) => {
  return (
    <div class="w-80 h-40 relative group bg-brand-2 rounded-lg bg-gradient-to-r from-cyan-300 to-blue-300 cursor-pointer">
      <div class="absolute bottom-0 px-4 py-2">
        <div class="font-medium text-brand-11 group-hover:text-brand-12">
          {props.title}
        </div>
      </div>
    </div>
  );
};

const Apps = () => {
  const apps = [
    {
      id: "app1",
      title: "My first app",
      description: "",
      access: [],
    },
  ];
  return (
    <>
      <Title>Apps</Title>
      <div class="">
        <div class="mt-40 px-16">
          <For each={apps} fallback="No apps">
            {(app) => {
              return <App {...app} />;
            }}
          </For>
        </div>
      </div>
    </>
  );
};

export default Apps;
