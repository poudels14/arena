import { createMutationQuery } from "@portal/solid-query";

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
        class="px-4 py-1.5 w-20 font-semibold text-xs text-center select-none rounded text-white bg-indigo-500 hover:bg-indigo-600 cursor-pointer"
        onClick={() => {
          inputRef.click();
        }}
      >
        Upload
      </div>
      <div>
        <form action="/api/fs/upload" method="post" ref={formRef}>
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
    </div>
  );
};

export { Uploader };
