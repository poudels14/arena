import { createSelector } from "solid-js";
import { Assistants } from "./Assistants";
import { InlineIcon } from "@arena/components";

import HomeIcon from "@blueprintjs/icons/lib/esm/generated-icons/20px/paths/home";
import StoreIcon from "@blueprintjs/icons/lib/esm/generated-icons/20px/paths/box";
import { useAssistantContext } from "../AssistantContext";
import { useNavigate } from "@solidjs/router";

const Sidebar = () => {
  const navigate = useNavigate();
  const { state } = useAssistantContext();

  const selectedTab = () => {
    const assistId = state.activeAssistantId();
    const activeTab = state.activeTab();
    const tab = activeTab == "chat" ? "" : "/" + activeTab;
    return "/assistants/" + assistId + tab;
  };
  const isSelected = createSelector(selectedTab);

  return (
    <div class="w-48 h-screen text-sm bg-accent-1 shadow">
      <div class="py-4 space-y-1 text-xs font-medium text-gray-600">
        <SidebarTab
          name="Home"
          iconPath={HomeIcon[0]}
          isSelected={isSelected}
          setSelected={() => navigate("/")}
        />
        <SidebarTab
          name="Store"
          iconPath={StoreIcon[0]}
          isSelected={isSelected}
          setSelected={() => navigate("/store")}
        />
      </div>
      <Assistants isSelected={isSelected} setSelected={navigate} />
    </div>
  );
};

const SidebarTab = (props: {
  name: string;
  iconPath: any;
  isSelected: (id: string) => boolean;
  setSelected: () => void;
}) => {
  return (
    <div
      class="px-2 py-1 table cursor-pointer hover:text-gray-800"
      classList={{
        "text-accent-12": props.isSelected(props.name),
      }}
      onClick={() => props.setSelected()}
    >
      <InlineIcon size="12px" class="table-cell">
        <path d={props.iconPath} />
      </InlineIcon>
      <div class="table-cell pl-2 w-full">{props.name}</div>
    </div>
  );
};

export { Sidebar };
