import { createSignal } from "solid-js";
import { HiOutlineArrowUpOnSquare, HiOutlinePlus } from "solid-icons/hi";
import { createMutationQuery } from "@portal/solid-query";
import Dialog from "@portal/solid-ui/Dialog";
import { Form, Input } from "@portal/solid-ui/form";
import { useUploadTrackerContext } from "./UploadTracker";

const Uploader = (props: {
  parentId: string | null;
  onUpload: (files: any[]) => void;
  onNewDirectory: () => void;
}) => {
  const { trackFileUpload } = useUploadTrackerContext();
  const uploader = createMutationQuery<any>((input) => {
    return {
      url: "/api/fs/upload",
      request: {
        form: input.body,
      },
    };
  }, {});

  const [createDirectorDialogVisible, showCreateDirecotryDialog] =
    createSignal(false);
  const createNewDirectory = createMutationQuery<{
    parentId: string | null;
    name: string;
  }>((input) => {
    return {
      url: "/api/fs/directory/add",
      request: {
        body: {
          parentId: input.parentId,
          name: input.name,
        },
      },
    };
  }, {});
  let formRef: any, inputRef: any;
  return (
    <div class="uploader p-2">
      <div class="flex divide-x divide-indigo-400">
        <div>
          <div
            class="flex px-2 py-1.5 space-x-1 font-semibold text-xs text-center select-none rounded-l text-white bg-indigo-500 hover:bg-indigo-600 cursor-pointer"
            onClick={() => {
              showCreateDirecotryDialog(true);
            }}
          >
            <HiOutlinePlus class="pt-0.5" />
            <div>New folder</div>
          </div>
        </div>
        <div
          class="flex px-2 py-1.5 space-x-1 font-semibold text-xs text-center select-none rounded-r text-white bg-indigo-500 hover:bg-indigo-600 cursor-pointer"
          onClick={() => {
            inputRef.click();
          }}
        >
          <HiOutlineArrowUpOnSquare class="pt-0.5" />
          <div>Upload file</div>
        </div>
      </div>
      <Dialog
        title={() => (
          <div class="title px-4 py-3 w-full font-medium text-xl text-center text-gray-700 border-b border-gray-100">
            Create a directory
          </div>
        )}
        open={createDirectorDialogVisible()}
        onOpenChange={(open) => showCreateDirecotryDialog(open)}
      >
        <div class="px-8 py-2 w-[550px] text-sm">
          <Form
            class="pt-4"
            onSubmit={(value) => {
              createNewDirectory
                .mutate({
                  parentId: props.parentId,
                  name: value.directoryName,
                })
                .then(() => {
                  props.onNewDirectory();
                  showCreateDirecotryDialog(false);
                });
            }}
          >
            <Input
              name="directoryName"
              placeholder="Directory name"
              class="px-3 py-2 w-full rounded-md"
            />
            <div class="pt-4 flex text-sm space-x-4 justify-end">
              <div
                class="px-4 py-1.5 rounded cursor-pointer hover:bg-gray-100"
                onClick={() => showCreateDirecotryDialog(false)}
              >
                Cancel
              </div>
              <button class="px-4 py-1.5 text-white bg-indigo-500 hover:bg-indigo-600 rounded cursor-pointer">
                Create
              </button>
            </div>
          </Form>
        </div>
      </Dialog>
      <form action="/api/fs/upload" method="post" ref={formRef} class="hidden">
        <input
          type="file"
          name="file"
          class="hidden"
          ref={inputRef}
          onChange={async () => {
            const formData = new FormData(formRef);
            formData.set("parentId", props.parentId || "null");

            const filename = (formData.get("file") as File).name;
            const tracking = trackFileUpload({
              title: filename,
            });
            await uploader
              .mutate({
                body: formData,
              })
              .then((res) => {
                if (!res.ok) {
                  tracking.error("Error uploading");
                }
              });

            props.onUpload(uploader.data().files);
            formRef.reset();
          }}
        />
      </form>
    </div>
  );
};

export { Uploader };
