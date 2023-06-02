import { A } from "@solidjs/router";

const Tab = (props: { title: string; href: string }) => {
  return (
    <A
      href={props.href}
      class="block px-1 py-1 rounded cursor-pointer hover:bg-brand-2"
      activeClass="bg-brand-4"
      end
    >
      {props.title}
    </A>
  );
};

const Tabs = () => {
  return (
    <div class="select-none">
      <div class="text-sm text-brand-12">
        {/* <Tab title="Home" href="/" /> */}
        <Tab title="Apps" href="/apps" />
        {/* <Tab title="Queries" href="/queries" /> */}
        <Tab title="Resources" href="/resources" />
        {/* <Tab title="Scheduled jobs" href="/jobs" /> */}
      </div>

      {/* <div class="mt-10">
        <div class="mb-2 font-medium">Favorites</div>
        <Tab title="App 1" href="/apps/app1" />
      </div> */}
    </div>
  );
};

export { Tabs };
