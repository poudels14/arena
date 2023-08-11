import { For } from "solid-js";
import { InlineIcon } from "@arena/components";
import LinkIcon from "@blueprintjs/icons/lib/esm/generated-icons/20px/paths/link";
import UploadIcon from "@blueprintjs/icons/lib/esm/generated-icons/20px/paths/cloud-upload";
import { Document } from "../types";
import { useAppContext } from "@arena/sdk/app";

const SUPPORTED_FILE_TYPES = [
  ".md",
  ".pdf",
  "text/*",
  ".js",
  ".jsx",
  ".ts",
  ".tsx",
  ".toml",
  ".yaml",
  ".json",
];

const Documents = (props: { documents: Document[] }) => {
  return (
    <>
      <div class="flex px-2 text-sm font-medium text-gray-800">
        <div class="flex-1 leading-6">Documents</div>
        <LinkNewDocument />
      </div>
      <div>
        <For each={props.documents}>
          {(document) => <Document id={document.id} name={document.name} />}
        </For>
      </div>
    </>
  );
};

const LinkNewDocument = () => {
  let fileInput: any;
  return (
    <div class="flex">
      <a
        href="#upload"
        onClick={(e) => {
          e.preventDefault();
          fileInput.click();
        }}
      >
        <InlineIcon
          size="24px"
          class="p-1.5 rounded cursor-pointer hover:bg-brand-11/20 text-brand-12/80"
        >
          <path d={UploadIcon[0]} />
        </InlineIcon>
      </a>
      <div class="hidden">
        <DocumentUploader ref={fileInput} />
      </div>
      <a
        href="#link"
        onClick={(e) => {
          e.preventDefault();
        }}
      >
        <InlineIcon
          size="24px"
          class="p-1.5 rounded cursor-pointer hover:bg-brand-11/20 text-brand-12/80"
        >
          <path d={LinkIcon[0]} />
        </InlineIcon>
      </a>
    </div>
  );
};

const DocumentUploader = (props: { ref: any }) => {
  const { router } = useAppContext();

  let formRef: any;
  const uploadDocument = () => {
    throw new Error("not implemented");
    // const formData = new FormData(formRef);
    // router.post("", formData, {
    //   headers: {
    //     "Content-Type": "multipart/form-data",
    //   },
    // });
  };

  return (
    <form
      ref={formRef}
      onSubmit={(e) => {
        e.preventDefault();
        e.stopPropagation();
        uploadDocument();
      }}
    >
      <input
        ref={props.ref}
        class="hidden"
        type="file"
        name="files"
        multiple={true}
        accept={SUPPORTED_FILE_TYPES.join(",")}
        onChange={() => {
          uploadDocument();
        }}
      >
        NICE!
      </input>
    </form>
  );
};

const Document = (props: { id: string; name: string; active?: boolean }) => {
  return (
    <label class="flex align-middle items-center">
      <div class="group relative">
        {/* <input
          type="checkbox"
          checked={props.active}
          disabled
          class="peer/check w-5 h-5 opacity-0 hidden cursor-pointer border border-red-200"
        /> */}
        {/* <div class="relative w-3 h-3 rounded bg-brand-5 peer-checked/check:bg-brand-11">
          <div class="absolute top-px left-1 w-1 h-2 border-gray-100 border-l-0 border-t-0 border-b-2 border-r-2 rotate-45"></div>
        </div> */}
      </div>
      <div
        class="flex-1 py-0.5 px-2 rounded cursor-pointer text-accent-12/80 hover:bg-accent-4"
        classList={
          {
            // "text-accent-9": !props.active,
          }
        }
      >
        {props.name}
      </div>
    </label>
  );
};

export { Documents };
