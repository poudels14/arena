import {
  For,
  Match,
  Show,
  Switch,
  createComputed,
  createSelector,
} from "solid-js";
import { Sidebar as PortalSidebar, SidebarTab } from "@portal/solid-ui/sidebar";
import { useMatcher, useNavigate } from "@portal/solid-router";
import { createQuery } from "@portal/solid-query";
import { HiOutlinePlus } from "solid-icons/hi";
import { Profile } from "./profile";
import { AddProfile } from "./add";

const Profiles = () => {
  const navigate = useNavigate();

  const tabMatcher = useMatcher(() => "/:profileId/*");
  const activeProfileId = () => tabMatcher()?.params?.profileId;
  const isProfileActive = createSelector(() => activeProfileId() || "-");

  const profiles = createQuery<any[]>(
    () => {
      return "/chat/profiles";
    },
    {},
    {
      lazy: true,
    }
  );
  createComputed(() => {
    profiles.refresh();
  });

  const gotoProfile = (id: string) => navigate(`/${id}`);

  return (
    <Show when={profiles.data()}>
      <div class="flex-1 flex">
        <div class="w-[225px] flex flex-col space-y-0">
          <div class="py-1 text-sm text-center bg-gray-100">Profiles</div>
          <div
            class="flex px-4 py-1.5 justify-start items-center cursor-pointer space-x-2 text-xs text-white bg-indigo-500 hover:bg-indigo-600"
            onClick={() => {
              navigate("/add");
            }}
          >
            <HiOutlinePlus />
            <div>Add new</div>
          </div>
          <PortalSidebar class="flex-1 basis-[225px] max-w-[225px] shrink-0 text-xs no-scrollbar pb-4 h-screen border-r border-gray-100 tab:px-4 tab:py-2 tab:text-gray-600 tab-hover:text-gray-700 tab-active:text-black tab-active:bg-gray-100 tab-active:font-medium icon:w-4 icon:h-4 icon:text-gray-400 overflow-y-auto">
            <For each={profiles.data()}>
              {(profile) => {
                return (
                  <SidebarTab
                    active={isProfileActive(profile.id)}
                    onClick={() => gotoProfile(profile.id)}
                  >
                    <div>{profile.name}</div>
                  </SidebarTab>
                );
              }}
            </For>
          </PortalSidebar>
        </div>
        <div class="flex-1 flex justify-center">
          <div class="flex-1 basis-[200px] max-w-[500px] py-5 px-5 text-gray-700">
            <Switch>
              <Match when={activeProfileId() && activeProfileId() != "add"}>
                <Profile
                  id={activeProfileId()!}
                  onDelete={() => {
                    profiles.refresh();
                    navigate("/");
                  }}
                />
              </Match>
              <Match when={activeProfileId() == "add"}>
                <AddProfile
                  onProfileAdd={(id) => {
                    profiles.refresh();
                    gotoProfile(id);
                  }}
                />
              </Match>
              <Match when={true}>
                <div class="flex h-full items-center">
                  <div class="text-gray-700">
                    Select a chat profile or add new one
                  </div>
                </div>
              </Match>
            </Switch>
          </div>
        </div>
      </div>
    </Show>
  );
};

export default Profiles;
