import { Input } from "@arena/components/form";
import Dialog from "@arena/components/Dialog";
import { Form, Textarea } from "@arena/components/form";
import { useDashboardContext } from "~/context";

const CreateAppDialog = (props: { closeDialog: () => void }) => {
  const { client, workspace } = useDashboardContext();
  return (
    <Dialog
      title="Create a new app"
      open={true}
      onOpenChange={(open) => !open && props.closeDialog()}
      contentClass="pt-5 px-8 w-[800px] shadow-accent-11"
    >
      <Form
        onSubmit={async (value) => {
          await client.apps.add.mutate({
            ...value,
            workspaceId: workspace.id,
          });
          props.closeDialog();
        }}
        class="w-full px-2 py-4 max-h-[350px] text-sm text-accent-12 overflow-y-auto space-y-5 no-scrollbar"
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

        <div class="flex justify-end">
          <button class="px-4 py-1.5 text-sm text-center text-accent-1 bg-brand-12/80 rounded">
            Submit
          </button>
        </div>
      </Form>
    </Dialog>
  );
};

export default CreateAppDialog;
