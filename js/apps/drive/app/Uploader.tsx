import { createMutationQuery } from "@portal/solid-query";
import { HiOutlineArrowUpOnSquare } from "solid-icons/hi";

const Uploader = (props: {
  parentId: string | null;
  onUpload: (files: any[]) => void;
}) => {
  const uploader = createMutationQuery<any>((input) => {
    return {
      url: "/api/fs/upload",
      request: {
        form: input.body,
      },
    };
  }, {});
  let formRef: any, inputRef: any;
  return (
    <div class="p-2">
      <div
        class="flex px-4 py-1.5 space-x-1 font-semibold text-xs text-center select-none rounded text-white bg-indigo-500 hover:bg-indigo-600 cursor-pointer"
        onClick={() => {
          inputRef.click();
        }}
      >
        <HiOutlineArrowUpOnSquare class="pt-0.5" />
        <div>Upload</div>
      </div>
      <form action="/api/fs/upload" method="post" ref={formRef} class="hidden">
        <input
          type="file"
          name="file"
          class="hidden"
          ref={inputRef}
          onChange={async () => {
            const formData = new FormData(formRef);
            formData.set("parentId", props.parentId || "null");
            await uploader.mutate({
              body: formData,
            });

            props.onUpload(uploader.data().files);
            // TODO: if error, show toast
            formRef.reset();
          }}
        />
      </form>
    </div>
  );
};

export { Uploader };
