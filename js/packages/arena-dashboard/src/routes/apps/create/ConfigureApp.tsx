import { createSignal, Show } from "solid-js";
import { Input } from "@arena/components/form";
import { Form, Textarea } from "@arena/components/form";
import { createMutationQuery } from "@arena/uikit/solid";
import type { TemplateManifest } from "@arena/sdk/app";
import { useDashboardContext } from "~/context";

const ConfigureApp = (props: {
  template: TemplateManifest;
  onCreate: () => void;
}) => {
  const { client, workspace } = useDashboardContext();
  const [error, setError] = createSignal<any | undefined>(undefined);
  const createNewApp = createMutationQuery<(value: any) => Promise<void>>(
    async (value) => {
      await client.apps.add.mutate({
        workspaceId: workspace.id,
        name: value.name,
        description: value.description,
        template: props.template,
      });
      props.onCreate();
    },
    {
      onError: setError,
    }
  );

  return (
    <Form
      onSubmit={(value) => createNewApp(value)}
      onChange={() => setError(undefined)}
      class="w-full pt-4 max-h-[350px] text-sm text-accent-12 overflow-y-auto space-y-5 no-scrollbar"
    >
      <div class="space-y-1">
        <label class="block font-medium">Name</label>
        <Input
          name="name"
          class="w-full"
          placeholder="Name"
          value={props.template.name}
        />
      </div>

      <div class="space-y-1">
        <label class="block text-base font-medium">Description</label>
        <Textarea
          name="description"
          rows={3}
          class="w-full"
          placeholder="App description"
        />
      </div>

      <div class="flex">
        <div class="flex-1 font-medium text-red-600">
          <Show when={error()}>{formatError(error())}</Show>
        </div>
        <button class="px-4 py-1.5 text-sm text-center text-accent-1 bg-brand-12/80 rounded">
          Submit
        </button>
      </div>
    </Form>
  );
};

const formatError = (error: any) => {
  const message = JSON.parse(error.message);
  if (message?.[0]?.code == "invalid_type") {
    return "Error creating a new app. Please check the values and try again.";
  }
  return "Error creating a new app";
};

export default ConfigureApp;
