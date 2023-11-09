import { For, Match, Show, Switch, createSignal, useContext } from "solid-js";
import { createMutationQuery } from "@arena/uikit/solid";
import { InlineIcon } from "@arena/components";
import LinkIcon from "@blueprintjs/icons/lib/esm/generated-icons/20px/paths/link";
import UploadIcon from "@blueprintjs/icons/lib/esm/generated-icons/20px/paths/cloud-upload";
import EditIcon from "@blueprintjs/icons/lib/esm/generated-icons/20px/paths/edit";
import DeleteIcon from "@blueprintjs/icons/lib/esm/generated-icons/20px/paths/trash";
import { Document } from "../types";
import { ChatContext } from "../chat/ChatContext";

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

const Documents = () => {
  const { router, state, setState } = useContext(ChatContext)!;

  const updateDocumentName = (id: string, name: string) =>
    setState("documents", (prev) => {
      return prev!.map((d) => {
        if (d.id == id) {
          return {
            ...d,
            name,
          };
        }
        return d;
      });
    });

  const renameDocument = async (
    id: string,
    oldName: string,
    newName: string
  ) => {
    updateDocumentName(id, newName);
    await router
      .post(`/api/documents/${id}/edit`, {
        name: newName,
      })
      .catch((_) => {
        updateDocumentName(id, oldName);
      });
  };

  const deleteDocument = async (id: string) => {
    setState("documents", (prev) => {
      return prev!.filter((d) => d.id !== id);
    });
    await router.delete(`/api/documents/${id}/delete`);
  };

  return (
    <Show when={state.documents()}>
      <div class="flex px-2 text-sm font-medium text-gray-800">
        <div class="flex-1 leading-6">Documents</div>
        <LinkNewDocument />
      </div>
      <div>
        <Switch>
          <Match when={state.documents()!.length > 0}>
            <For each={state.documents()}>
              {(document) => {
                return (
                  <DocumentTab
                    {...document}
                    renameDocument={renameDocument}
                    deleteDocument={deleteDocument}
                  />
                );
              }}
            </For>
          </Match>
          <Match when={true}>
            <div class="text-xs text-center text-gray-400">
              No documents linked yet
            </div>
          </Match>
        </Switch>
      </div>
    </Show>
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
  const { router, setState } = useContext(ChatContext)!;

  let formRef: any;
  const uploadDocument = createMutationQuery(async () => {
    const formData = new FormData(formRef);
    const res = await router.post("/api/documents/upload", formData, {
      headers: {
        "Content-Type": "multipart/form-data",
      },
    });

    setState("documents", (prev) => {
      const { existing: existingDocs, new: newDocs } = res.data;
      const unique: Document[] = [...(prev || [])];
      existingDocs.forEach((d: Document) => {
        if (!unique.find((u) => u.id === d.id)) {
          unique.push(d);
        }
      });
      newDocs.forEach((d: Document) => {
        if (!unique.find((u) => u.id === d.id)) {
          unique.push({
            ...d,
            isNew: true,
          });
        }
      });
      return unique;
    });

    formRef.reset();
  });

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

const DocumentTab = (props: {
  id: string;
  name: string;
  isNew?: boolean;
  active?: boolean;
  renameDocument: (id: string, oldName: string, newName: string) => void;
  deleteDocument: (id: string) => void;
}) => {
  const [isEditMode, setEditMode] = createSignal(false);
  const renameDocument = (e: any) => {
    props.renameDocument(props.id, props.name, e.target.value);
    setEditMode(false);
  };
  return (
    <label class="group flex align-middle items-center">
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
        class="flex-1 py-0.5 px-2 rounded cursor-pointer text-accent-12/80 text-ellipsis hover:bg-accent-4"
        classList={{
          "bg-brand-12/10": isEditMode(),
        }}
      >
        <Switch>
          <Match when={isEditMode()}>
            <input
              class="w-full bg-transparent outline-none"
              value={props.name}
              onChange={renameDocument}
            />
          </Match>
          <Match when={true}>{props.name}</Match>
        </Switch>
      </div>
      <Show when={props.isNew}>
        <div class="w-1.5 h-1.5 bg-green-500 rounded-full" />
      </Show>
      <div class="flex flex-row">
        <Show when={!isEditMode()}>
          <InlineIcon
            size="18px"
            class="hidden group-hover:block p-1 rounded cursor-pointer hover:bg-brand-11/10 text-brand-12/80"
            onClick={() => setEditMode(true)}
          >
            <path d={EditIcon[0]} />
          </InlineIcon>
          <InlineIcon
            size="18px"
            class="hidden group-hover:block p-1 rounded cursor-pointer hover:bg-brand-11/10 text-brand-12/80"
            onClick={() => props.deleteDocument(props.id)}
          >
            <path d={DeleteIcon[0]} />
          </InlineIcon>
        </Show>
      </div>
    </label>
  );
};

export { Documents };
