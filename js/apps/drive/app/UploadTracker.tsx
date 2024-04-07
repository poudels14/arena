import { For, Match, Show, Switch, createContext, useContext } from "solid-js";
import { createStore } from "@portal/solid-store";
import Spinner from "@portal/solid-ui/Spinner";
import { HiOutlineCheckCircle } from "solid-icons/hi";

type UploadTrackerContext = {
  trackFileUpload: (options: { title: string }) => {
    remove(): void;
    success(): void;
    error(msg: string): void;
  };
};
const UploadTrackerContext = createContext<UploadTrackerContext>();
const useUploadTrackerContext = () => useContext(UploadTrackerContext)!;

type State = {
  uploads: UploadStatus[];
};

type UploadStatus = {
  id: number;
  title: string;
  success?: boolean;
  error?: string;
};

let uploadTrackingId = 1;
const UploadTrackerProvider = (props: { children: any }) => {
  const [state, setState] = createStore<State>({
    uploads: [],
  });

  const trackFileUpload: UploadTrackerContext["trackFileUpload"] = (
    options
  ) => {
    const id = uploadTrackingId++;
    setState("uploads", (prev) => {
      return [
        ...prev,
        {
          id,
          title: options.title,
        },
      ];
    });

    function updateUploadState(updater: (prev: UploadStatus) => UploadStatus) {
      setState("uploads", (prev) => {
        return prev.map((upload) => {
          if (upload.id == id) {
            return updater(upload);
          } else {
            return upload;
          }
        });
      });
    }

    return {
      remove() {
        setState("uploads", (prev) => prev.filter((u) => u.id != id));
      },
      success() {
        updateUploadState((prev) => {
          return {
            ...prev,
            success: true,
          };
        });
      },
      error(msg: string) {
        // remove tracking after some time
        setTimeout(() => {
          this.remove();
        }, 30_000);
        updateUploadState((prev) => {
          return {
            ...prev,
            error: msg,
          };
        });
      },
    };
  };
  return (
    <UploadTrackerContext.Provider
      value={{
        // @ts-expect-error
        state,
        trackFileUpload,
      }}
    >
      {props.children}
    </UploadTrackerContext.Provider>
  );
};

const UploadTracker = (props: {}) => {
  const {
    // @ts-expect-error
    state,
  } = useUploadTrackerContext();
  return (
    <Show when={state.uploads().length > 0}>
      <div class="fixed bottom-0 right-2 w-[28rem] text-sm rounded-t border border-gray-200 bg-gray-50">
        <div class="w-full px-2 py-2 text-center font-medium rounded-t border-b text-gray-800 border-gray-200 bg-gray-100">
          Uploads
        </div>
        <div class="px-2 py-2 max-h-40 overflow-scroll no-scrollbar divide-y divide-gray-100">
          <For each={state.uploads()}>
            {(upload) => {
              return (
                <div class="flex items-center py-1 space-x-2">
                  <div>{upload.title}</div>
                  <Switch>
                    <Match when={upload.success}>
                      <HiOutlineCheckCircle
                        size={14}
                        class="pb-px text-green-700"
                      />
                    </Match>
                    <Match when={upload.error}>
                      <div class="max-w-52 overflow-hidden text-ellipsis whitespace-nowrap text-xs text-red-700">
                        {upload.error}
                      </div>
                    </Match>
                    <Match when={true}>
                      <div class="h-3 pb-px">
                        <Spinner />
                      </div>
                    </Match>
                  </Switch>
                </div>
              );
            }}
          </For>
        </div>
      </div>
    </Show>
  );
};

export { UploadTrackerProvider, useUploadTrackerContext, UploadTracker };
