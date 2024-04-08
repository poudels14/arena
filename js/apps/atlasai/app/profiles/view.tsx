import { Match, Switch, createComputed } from "solid-js";
import { createMutationQuery, createQuery } from "@portal/solid-query";

type Profile = {
  id: string;
  name: string;
  description: string;
  prompt: string;
  default: boolean;
};

const ViewProfile = (props: {
  id: string;
  refreshProfiles: () => void;
  editProfile: (profileId: string) => void;
  onDelete: (id: string) => void;
}) => {
  const profile = createQuery<{
    name: string;
    description: string;
    template: string;
    default: boolean;
  }>(
    () => {
      return `/chat/profiles/${props.id}`;
    },
    {},
    {
      lazy: true,
    }
  );
  // refresh profile every time id changes
  createComputed(() => {
    void props.id;
    profile.refresh();
  });

  const updateProfile = createMutationQuery<{
    default?: boolean;
  }>((input) => {
    return {
      url: `/chat/profiles/${props.id}/update`,
      request: {
        body: input,
      },
    };
  });

  const deleteProfile = createMutationQuery<{}>(() => {
    return {
      url: `/chat/profiles/${props.id}/delete`,
      request: {
        method: "POST",
      },
    };
  });

  return (
    <div>
      <Switch>
        <Match when={profile.status() == 404}>
          <div>Not found</div>
        </Match>
        <Match when={profile.data()}>
          <div class="space-y-6">
            <div class="space-y-1.5">
              <div class="text-base font-medium text-gray-800">Name</div>
              <div class="text-sm">{profile.data.name()}</div>
            </div>
            <div class="space-y-1.5">
              <div class="text-base font-medium text-gray-800">Description</div>
              <div class="text-sm">
                {profile.data.description() || (
                  <i class="font-light text-gray-500">
                    No description available
                  </i>
                )}
              </div>
            </div>
            <div class="space-y-1.5">
              <div class="text-base font-medium text-gray-800">Prompt</div>
              <div class="text-sm">{profile.data.template!()}</div>
            </div>
            <div class="space-y-1.5">
              <Switch>
                <Match when={profile.data.default()}>
                  <div class="px-3 py-2 text-xs text-indigo-600 text-center rounded border">
                    This is the default profile
                  </div>
                </Match>
                <Match when={true}>
                  <button
                    type="button"
                    class="px-4 py-1.5 text-xs text-indigo-800 rounded border border-indigo-100 bg-indigo-50 hover:bg-indigo-200"
                    onClick={() => {
                      updateProfile
                        .mutate({
                          default: true,
                        })
                        .then(() => {
                          profile.refresh();
                        });
                    }}
                  >
                    Set as default
                  </button>
                </Match>
              </Switch>
            </div>
            <div class="space-x-4">
              <button
                type="button"
                class="px-6 py-1.5 text-xs text-indigo-800 rounded border border-indigo-100 bg-indigo-50 hover:bg-indigo-200"
                onClick={() => props.editProfile(props.id)}
              >
                Edit Profile
              </button>
              <button
                type="button"
                class="px-6 py-1.5 text-xs text-red-800 rounded border border-red-100 bg-red-50 hover:bg-red-200"
                onClick={() => {
                  deleteProfile.mutate({}).then(() => {
                    props.onDelete(props.id);
                  });
                }}
              >
                Delete
              </button>
            </div>

            {/* <button
              type="button"
              class="px-6 py-1.5 text-xs text-white rounded bg-gray-500 hover:bg-red-500"
              onClick={() => {
                deleteProfile.mutate({}).then(() => {
                  props.onDelete(props.id);
                });
              }}
            >
              Delete
            </button> */}
          </div>
        </Match>
      </Switch>
    </div>
  );
};

export { ViewProfile };
export type { Profile };
