import {
  Router,
  Route,
  useNavigate,
  useMatcher,
  useLocation,
} from "@portal/solid-router";
import { Sidebar as PortalSidebar, SidebarTab } from "@portal/solid-ui/sidebar";
import { createEffect, createSelector } from "solid-js";

const AgentWorkflows = () => {
  return (
    <div>
      <div class="flex justify-center">
        <div class="flex-1 max-w-[700px] space-y-4">
          <Workflow
            id="1"
            title="Generate QR code"
            description="This Agent will generate QR code"
          />
          <Workflow
            id="1"
            title="Order from Doordash"
            description="Order doordash"
          />
          <Workflow
            id="1"
            title="Show my active Github PRs"
            description="Active Github PRs"
          />
        </div>
      </div>
    </div>
  );
};

type WorkflowProps = {
  id: string;
  title: string;
  description: string;
};

const Workflow = (props: WorkflowProps) => {
  return (
    <div class="px-3 py-2 text-sm border border-gray-100 bg-gray-50 rounded space-y-1 cursor-pointer hover:border-gray-200 hover:bg-gray-100">
      <div class="font-medium">{props.title}</div>
      <div class="text-xs text-gray-700">{props.description}</div>
    </div>
  );
};

export default AgentWorkflows;
