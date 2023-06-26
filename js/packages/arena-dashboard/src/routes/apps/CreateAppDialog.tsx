import { createSignal, Show } from "solid-js";
import { Input } from "@arena/components/form";
import Dialog from "@arena/components/Dialog";
import { Form, Textarea } from "@arena/components/form";
import { createMutationQuery } from "@arena/uikit/solid";
import { useDashboardContext } from "~/context";

const CreateAppDialog = (props: { closeDialog: () => void }) => {
  const { client, workspace } = useDashboardContext();
  const [error, setError] = createSignal<any | undefined>(undefined);
  const createNewApp = createMutationQuery<any>(
    async (value) => {
      await client.apps.add.mutate({
        ...value,
        workspaceId: workspace.id,
      });
      props.closeDialog();
    },
    {
      onError: setError,
    }
  );

  return (
    <Dialog
      title="Create a new app"
      open={true}
      onOpenChange={(open) => !open && props.closeDialog()}
      contentClass="pt-4 px-4 w-[800px] shadow-accent-11"
    >
      <Form
        onSubmit={(value) => createNewApp(value)}
        onChange={() => setError(undefined)}
        class="w-full pt-4 max-h-[350px] text-sm text-accent-12 overflow-y-auto space-y-5 no-scrollbar"
      >
        <div class="space-y-1">
          <label class="block font-medium">Name</label>
          <Input name="name" class="w-full" placeholder="Name" />
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
    </Dialog>
  );
};

const formatError = (error: any) => {
  const message = JSON.parse(error.message);
  if (message?.[0]?.code == "invalid_type") {
    return "Error creating a new app. Please check the values and try again.";
  }
  return "Error creating a new app";
};

export default CreateAppDialog;
