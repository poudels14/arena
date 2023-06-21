import { useDashboardContext } from "~/context";

const WorkspacePanel = () => {
  const { workspace } = useDashboardContext();
  return (
    <div class="">
      <div class="font-medium py-2">{workspace.name}</div>
    </div>
  );
};

export { WorkspacePanel };
