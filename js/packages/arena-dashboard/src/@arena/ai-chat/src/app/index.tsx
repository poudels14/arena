import { Sidebar } from "./Sidebar";
import { Show, createResource, createSignal } from "solid-js";
import { useAppContext } from "@arena/sdk/app";
import { Document } from "./types";

async function* textStream() {
  let count = 0;
  while (count++ < 1000) {
    yield new Promise((resolve) => {
      setTimeout(() => resolve("text "), 0);
    });
  }
}

const App = (props: any) => {
  const { router } = useAppContext();
  const [getDocuments] = createResource(() =>
    router.query<Document[]>("/documents")
  );
  const [getValue, setValue] = createSignal("");

  async function main() {
    const stream = textStream();
    for await (const value of stream) {
      setValue((prev) => prev + value);
    }
  }

  main();

  return (
    <Show when={getDocuments()}>
      <div class="w-full h-screen flex flex-row">
        <Sidebar documents={getDocuments()!} />
        <div class="relative flex-1 flex flex-col">
          <div class="flex-1 px-5 flex justify-center overflow-y-auto">
            <div class="flex-1 max-w-[650px] h-full">
              <div class="pb-16">
                <div>
                  <div class="chat-response text-accent-12/80">
                    {getValue()}
                  </div>
                </div>
              </div>
            </div>
          </div>
          <div class="absolute bottom-2 w-full flex justify-center">
            <div class="flex-1 px-5 max-w-[560px]">
              <Chatbox />
            </div>
          </div>
        </div>
      </div>
    </Show>
  );
};

const Chatbox = () => {
  const [getValue, setValue] = createSignal("");
  function calcHeight(value: any) {
    let numberOfLineBreaks = (value.match(/\n/g) || []).length;
    // min-height + lines x line-height + padding + border
    let newHeight = 20 + numberOfLineBreaks * 20 + 24 + 2;
    return newHeight + "px";
  }

  return (
    <textarea
      placeholder="Send a message"
      class="px-4 py-3 w-full max-h-[180px] rounded-lg text-sm text-white bg-brand-12/90 shadow-lg backdrop-blur-sm outline-none resize-none placeholder:text-gray-400"
      style={{
        height: calcHeight(getValue()),
        "--uikit-scrollbar-track-bg": "transparent",
        "--uikit-scrollbar-track-thumb": "rgb(210, 210, 210)",
      }}
      value={getValue()}
      onInput={(e) => setValue(e.target.value)}
    />
  );
};

export default App;
