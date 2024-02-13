import { createQuery } from "@portal/solid-query";
import { For, Match, Show, Switch, createMemo, createSignal } from "solid-js";

type InterpreterState = {
  result: any;
};

const Interpreter = (props: {
  metadata: any;
  state: InterpreterState;
  terminated: boolean;
  UI: {
    Markdown: any;
    Table: any;
  };
}) => {
  const { Markdown } = props.UI;
  const result = createMemo(() => props.state.result || {});
  const artifacts = createMemo(() => {
    return props.state.result?.artifacts;
  });
  return (
    <div class="py-2 space-y-4">
      <div class="space-y-1">
        <div class="font-bold">Using code Interpreter</div>
        <div class="max-h-[150px] overflow-y-scroll scroll:w-1 thumb:rounded thumb:bg-gray-400">
          <Markdown
            markdown={"```python\n" + props.metadata.arguments.code + "\n```"}
          />
        </div>
      </div>
      <Show when={result().error?.length > 0}>
        <div class="space-y-0.5">
          <div class="font-semibold text-red-600">Error running code</div>
          <div class="px-2 py-3 rounded bg-gray-100 text-gray-700">
            {result().error}
          </div>
        </div>
      </Show>
      <Show when={result().stdout?.length > 0}>
        <Markdown markdown={"```bash\n" + result().stdout + "\n```"} />
      </Show>
      <Show when={artifacts()?.length > 0}>
        <Artifacts UI={props.UI} artifacts={artifacts()} />
      </Show>
    </div>
  );
};

const Artifacts = (props: { UI: any; artifacts: any }) => {
  const { Markdown, Table } = props.UI;
  const [getActiveTab, setActiveTab] = createSignal(0);

  const artifactContent = createQuery<any>(() => {
    const index = getActiveTab();
    return `/chat/artifacts/${props.artifacts[index].id}/content?json=true`;
  }, {});

  const contentType = createMemo(() => artifactContent.data()?.contentType);
  const data = createMemo(() => artifactContent.data()?.data);
  return (
    <div class="space-y-1">
      <div class="font-bold">Output files</div>
      <div class="space-y-0">
        <div class="flex justify-start space-x-1">
          <For each={props.artifacts}>
            {(file, index) => {
              return (
                <div
                  class="px-2 py-1 rounded-t bg-gray-200 cursor-pointer"
                  onClick={() => setActiveTab(index())}
                >
                  {file.name}
                </div>
              );
            }}
          </For>
        </div>

        <div>
          <Show when={artifactContent.data()}>
            <Switch>
              <Match when={contentType() == "text/plain"}>
                <Markdown markdown={"```txt\n" + data() + "```"} />
              </Match>
              <Match when={contentType() == "text/csv"}>
                <Table
                  rows={data().rows || []}
                  class="w-full"
                  downloadUrl="NICE!"
                />
              </Match>
            </Switch>
          </Show>
        </div>
      </div>
    </div>
  );
};

export default Interpreter;
