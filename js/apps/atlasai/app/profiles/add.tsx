import { createMutationQuery, createQuery } from "@portal/solid-query";
import { adjustTextareaHeight } from "@portal/solid-ui/form/Textarea";
import { createStore } from "@portal/solid-store";
import { Match, Switch, createComputed } from "solid-js";
import { Profile } from "./view";

const AddOrUpdateProfile = (props: {
  id?: string;
  onSuccess: (profileId: string) => void;
}) => {
  const [state, setState] = createStore({
    name: "",
    description: "",
    template: "",
    error: "",
  });

  const profile = createQuery<{
    name: string;
    description: string;
    template: string;
    default: boolean;
  }>(
    () => {
      if (!props.id) {
        return null;
      }
      return `/chat/profiles/${props.id}`;
    },
    {},
    {
      lazy: true,
    }
  );
  // refresh profile every time id changes
  createComputed(() => {
    if (props.id) {
      profile.refresh();
    }
  });
  createComputed(() => {
    const data = profile.data();
    if (data) {
      setState((prev) => {
        return {
          ...prev,
          ...data,
        };
      });
    }
  });

  const setField = (field: string, value: string) => {
    setState(field as any, value);
    setState("error", "");
  };

  const addProfile = createMutationQuery<{
    name: string;
    description: string;
    prompt: string;
  }>((input) => {
    return {
      url: `/chat/profiles/add`,
      request: {
        body: input,
      },
    };
  });

  const updateProfile = createMutationQuery<
    Pick<Profile, "id"> & Partial<Profile>
  >((input) => {
    const { id, ...body } = input;
    return {
      url: `/chat/profiles/${id}/update`,
      request: {
        body,
      },
    };
  });

  return (
    <div class="space-y-6">
      <div class="space-y-1.5">
        <div class="text-base font-medium text-gray-800">Name</div>
        <input
          type="text"
          name="name"
          placeholder="Name"
          class="w-full px-2 py-1.5 text-sm border border-gray-200 rounded outline-none focus:ring-1"
          onInput={(e) => setField("name", e.target.value)}
          value={state.name()}
        />
      </div>
      <div class="space-y-1.5">
        <div class="text-base font-medium text-gray-800">Description</div>
        <textarea
          name="description"
          placeholder="Description"
          class="w-full h-24 px-2 py-1.5 text-sm border border-gray-200 rounded outline-none focus:ring-1"
          ref={(node) =>
            adjustTextareaHeight(node, state.description, {
              defaultLines: 3,
              maxLines: 40,
            })
          }
          onInput={(e) => setField("description", e.target.value)}
          value={state.description()}
        ></textarea>
      </div>
      <div class="space-y-1.5">
        <div class="text-base font-medium text-gray-800">Prompt</div>
        <textarea
          name="prompt"
          placeholder="Prompt"
          class="w-full h-24 px-2 py-1.5 text-sm border border-gray-200 rounded outline-none focus:ring-1"
          ref={(node) =>
            adjustTextareaHeight(node, state.template, {
              defaultLines: 3,
              maxLines: 40,
            })
          }
          onInput={(e) => setField("template", e.target.value)}
          value={state.template()}
        ></textarea>
      </div>
      <div class="text-sm text-red-600 whitespace-pre">
        <div>{state.error() || " "}</div>
      </div>
      <div class="space-x-4">
        <Switch>
          <Match when={props.id}>
            <button
              type="button"
              class="px-6 py-1.5 text-xs text-white rounded bg-indigo-500 hover:bg-indigo-600"
              onClick={() => {
                updateProfile
                  .mutate({
                    id: props.id!,
                    name: state.name(),
                    description: state.description(),
                    prompt: state.template(),
                  })
                  .then(async (res) => {
                    if (!res.ok) {
                      setState("error", await res.text());
                    } else {
                      props.onSuccess(props.id!);
                    }
                  });
              }}
            >
              Update profile
            </button>
            <button
              type="button"
              class="px-6 py-1.5 text-xs text-gray-800 rounded border border-gray-100 bg-gray-50 hover:bg-gray-200"
              onClick={() => props.onSuccess(props.id!)}
            >
              Cancel
            </button>
          </Match>
          <Match when={!props.id}>
            <button
              type="button"
              class="px-6 py-1.5 text-xs text-white rounded bg-indigo-500 hover:bg-indigo-600"
              onClick={() => {
                addProfile
                  .mutate({
                    name: state.name(),
                    description: state.description(),
                    prompt: state.template(),
                  })
                  .then(async (res) => {
                    if (!res.ok) {
                      setState("error", await res.text());
                    } else {
                      const profile = addProfile.data();
                      props.onSuccess(profile.id);
                    }
                  });
              }}
            >
              Add profile
            </button>
          </Match>
        </Switch>
      </div>
    </div>
  );
};

export { AddOrUpdateProfile };
