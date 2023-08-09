import { Show, Switch, Match, createSignal } from "solid-js";
import { Input, Select } from "@arena/components/form";
import Dialog from "@arena/components/Dialog";
import { Form, Textarea } from "@arena/components/form";
import PostgresConfig from "./resources/Postgres";
import ApiKeyConfig from "./resources/ApiKey";
import { useDashboardContext } from "~/context";
import { createMutationQuery } from "@arena/uikit/solid";

const AddResourceDialog = (props: {
  closeDialog: () => void;
  resourceTypes: { id: string; name: string }[];
}) => {
  const { client, workspace } = useDashboardContext();
  const [resourceType, setResourceType] = createSignal<string>();
  const addNewResource = createMutationQuery<(value: any) => Promise<void>>(
    async (value) => {
      await client.resources.add.mutate({
        ...value,
        workspaceId: workspace.id,
      });
      props.closeDialog();
    }
  );

  return (
    <Dialog
      title="Add a new resource"
      open={true}
      onOpenChange={(open) => !open && props.closeDialog()}
      contentClass="pt-4 px-4 w-[800px] shadow-accent-11"
    >
      <Form
        onSubmit={addNewResource}
        class="w-full pt-4 h-[350px] text-sm text-accent-12 overflow-y-auto space-y-5 shadow-sm no-scrollbar"
      >
        <div class="space-y-1">
          <label class="block text-base font-medium">Resource type</label>
          <Select
            class="w-full text-sm"
            name="type"
            placeholder="Select resource type"
            options={props.resourceTypes}
            optionValue="id"
            optionTextValue="name"
            triggerClass="w-64"
            itemClass="text-sm"
            contentClass="w-64"
            onChange={setResourceType}
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
        <Show when={resourceType()}>
          <div class="space-y-2">
            <div class="mb-1 block text-base font-medium">Resource Config</div>
            <div class="space-y-3">
              <Switch>
                <Match when={resourceType() == "@arena/sql/postgres"}>
                  <PostgresConfig />
                </Match>
                <Match when={resourceType()?.startsWith("@arena/apikey/")}>
                  <ApiKeyConfig />
                </Match>
              </Switch>
            </div>
          </div>
        </Show>
        <div class="flex">
          <div class="flex-1 font-medium text-red-600">
            <Show when={addNewResource.error}>
              Error adding a new resource. Please try again.
            </Show>
          </div>
          <button class="px-4 py-1.5 text-sm text-center text-accent-1 bg-brand-12/80 rounded">
            Submit
          </button>
        </div>
      </Form>
    </Dialog>
  );
};

export default AddResourceDialog;
