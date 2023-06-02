import { Input, Select } from "@arena/components/form";
import Dialog from "@arena/components/Dialog";
import { Form, Textarea } from "@arena/components/form";
import PostgresConfig from "./resources/Postgres";
import { useDashboardContext } from "~/context";

const AddResource = (props: { closeDialog: () => void }) => {
  const { client, workspaceId } = useDashboardContext();
  return (
    <div class="w-full h-full p-10">
      <Dialog
        title="Add a new resource"
        open={true}
        onOpenChange={(open) => !open && props.closeDialog()}
        contentClass="pt-5 px-8 w-[800px] shadow-accent-11"
      >
        <Form
          onSubmit={async (value) => {
            await client.resources.add.mutate({
              ...value,
              workspaceId,
            });
            props.closeDialog();
          }}
          class="w-full px-2 py-4 h-[350px] text-sm text-accent-12 overflow-y-auto space-y-5 shadow-sm no-scrollbar"
        >
          <div class="space-y-1">
            <label class="block text-base font-medium">Resource type</label>
            <Select
              class="w-full text-sm"
              name="type"
              placeholder="Select resource type"
              options={[
                {
                  name: "@arena/sql/postgres",
                  title: "Postgres database",
                },
              ]}
              optionValue="name"
              optionTextValue="title"
              itemClass="text-sm"
            />
          </div>

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
              placeholder="Resource description"
            />
          </div>

          <div>
            <div class="mb-1 block text-base text-accent-11">
              Resource Config
            </div>

            <div class="space-y-3">
              <PostgresConfig />
            </div>
          </div>

          <div class="flex justify-end">
            <button class="px-4 py-1.5 text-sm text-center text-accent-1 bg-brand-12/80 rounded">
              Submit
            </button>
          </div>
        </Form>
      </Dialog>
    </div>
  );
};

export default AddResource;
