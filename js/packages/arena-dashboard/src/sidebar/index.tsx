import { Tabs } from "./Tabs";
import { WorkspacePanel } from "./WorkspacePanel";

const Sidebar = () => {
  return (
    <div class="h-full flex flex-col">
      <div class="px-4 bg-brand-12/90 text-white">
        <WorkspacePanel />
      </div>
      <div class="mt-8 flex-1">
        <Tabs />
      </div>
    </div>
  );
};

export { Sidebar };
