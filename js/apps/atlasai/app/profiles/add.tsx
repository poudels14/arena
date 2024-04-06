import { createMutationQuery } from "@portal/solid-query";
import { adjustTextareaHeight } from "@portal/solid-ui/form/Textarea";
import { createStore } from "@portal/solid-store";

const AddProfile = (props: { onProfileAdd: (profileId: string) => void }) => {
  const [state, setState] = createStore({
    name: "",
    description: "",
    prompt: "",
    error: "",
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
        ></textarea>
      </div>
      <div class="space-y-1.5">
        <div class="text-base font-medium text-gray-800">Prompt</div>
        <textarea
          name="prompt"
          placeholder="Prompt"
          class="w-full h-24 px-2 py-1.5 text-sm border border-gray-200 rounded outline-none focus:ring-1"
          ref={(node) =>
            adjustTextareaHeight(node, state.prompt, {
              defaultLines: 3,
              maxLines: 40,
            })
          }
          onInput={(e) => setField("prompt", e.target.value)}
        ></textarea>
      </div>
      <div class="text-sm text-red-600 whitespace-pre">
        <div>{state.error() || " "}</div>
      </div>
      <div class="space-y-1.5">
        <button
          type="button"
          class="px-6 py-1.5 text-xs text-white rounded bg-indigo-500 hover:bg-indigo-600"
          onClick={() => {
            addProfile
              .mutate({
                name: state.name(),
                description: state.description(),
                prompt: state.prompt(),
              })
              .then(async (res) => {
                if (!res.ok) {
                  setState("error", await res.text());
                } else {
                  const profile = addProfile.data();
                  props.onProfileAdd(profile.id);
                }
              });
          }}
        >
          Add profile
        </button>
      </div>
    </div>
  );
};

export { AddProfile };
