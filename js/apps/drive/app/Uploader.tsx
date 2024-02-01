const Uploader = (props: {
  parentId: string | null;
  onUpload: (files: any[]) => void;
}) => {
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
              const data = await fetch("/api/fs/upload", {
                method: "POST",
                body: formData,
              }).then((res) => res.json());
              props.onUpload(data.files);
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
