import {
  For,
  Switch,
  Match,
  Show,
  createReaction,
  createComputed,
} from "solid-js";
import { createStore } from "@portal/solid-store";
import Dialog from "@portal/solid-ui/Dialog";
import { createMutationQuery } from "@portal/solid-query";

const AddModel = (props: { workspaceId: string; closeDialog: () => void }) => {
  const [state, setState] = createStore({
    error: "",
    name: "",
    type: "chat",
    modalities: ["text"],
    provider: "",
    // eg: 'mistral'. used for provider like Ollama
    modelName: "",
    apiEndpoint: "",
    apiKey: "",
  });

  const resetError = createReaction(() => {
    setState("error", "");
  });
  // reset error if the state changes after the error is set
  createComputed(() => {
    void state.error();
    resetError(() => state());
  });

  const addModel = createMutationQuery(() => {
    return {
      url: "/api/llm/models/add",
      request: {
        body: {
          workspaceId: props.workspaceId,
          model: {
            name: state.name(),
            modalities: state.modalities(),
            type: state.type(),
            provider: state.provider(),
            config: {
              http: {
                apiKey: state.apiKey(),
                endpoint: state.apiEndpoint(),
              },
              model: {
                name: state.modelName(),
              },
            },
          },
        },
      },
    };
  });

  return (
    <Dialog
      title={() => (
        <div class="title px-8 pt-4 w-full font-medium text-lg text-left text-gray-700 border-gray-100">
          Add custom model
        </div>
      )}
      open={true}
      onOpenChange={props.closeDialog}
    >
      <div class="px-8 pb-4 w-[580px] text-sm space-y-4">
        <div>
          <label class="space-y-1.5">
            <div class="text-base font-medium text-gray-800">Name</div>
            <input
              type="text"
              name="model-name"
              placeholder="Name"
              class="w-full px-2 py-1.5 text-sm border border-gray-200 bg2-gray-200 rounded outline-none focus:ring-1"
              onInput={(e) => setState("name", e.target.value)}
            />
          </label>
        </div>
        <div class="space-y-2">
          <div class="text-base font-medium text-gray-800">Model Type</div>
          <div class="flex flex-wrap gap-4">
            <RadioOption
              id="text"
              title="Text"
              name="type"
              selected="text"
              onChange={(v) => {}}
            />
          </div>
        </div>
        <div class="space-y-2">
          <div class="text-base font-medium text-gray-800">Select provider</div>
          <div class="flex flex-wrap gap-4">
            <Switch>
              <Match when={state.type() == "chat"}>
                <TextModelProviders
                  value={state.provider()}
                  onChange={(provider) => {
                    if (provider == "ollama") {
                      setState("apiEndpoint", "http://localhost:11434");
                    } else if (provider == "lmstudio") {
                      setState("apiEndpoint", "http://localhost:1234/v1");
                    }
                    setState("provider", provider);
                  }}
                />
              </Match>
              <Match when={state.type() == "image"}>
                <ImageModelProviders
                  value={state.provider()}
                  onChange={(v) => setState("provider", v)}
                />
              </Match>
            </Switch>
          </div>
        </div>
        <Show when={state.provider() == "ollama"}>
          <div>
            <Show when={state.provider() == "ollama"}>
              <Input
                title="API Endpoint"
                name="apiEndpoint"
                placeholder="http://localhost:11434"
                type="text"
                value={state.apiEndpoint()}
                onInput={(apiEndpoint) => {
                  setState("apiEndpoint", apiEndpoint);
                }}
              />
            </Show>
          </div>
          <div>
            <Input
              title="Model name"
              name="llmModel"
              placeholder="mistral"
              type="text"
              value={state.modelName()}
              onInput={(modelName) => {
                setState("modelName", modelName);
              }}
            />
          </div>
        </Show>
        <Show when={state.provider() == "lmstudio"}>
          <div>
            <Input
              title="API Endpoint"
              name="apiEndpoint"
              placeholder="http://localhost:1234/v1"
              type="text"
              value={state.apiEndpoint()}
              onInput={(apiEndpoint) => {
                setState("apiEndpoint", apiEndpoint);
              }}
            />
          </div>
        </Show>
        <Show when={["openai", "anthropic", "groq"].includes(state.provider())}>
          <div>
            <Input
              title="API Key"
              name="apiKey"
              placeholder="API Key"
              type="text"
              value={state.apiKey()}
              onInput={(apiKey) => {
                setState("apiKey", apiKey);
              }}
            />
          </div>
          <div class="space-y-2">
            <div class="text-base font-medium text-gray-800">Model name</div>
            <div class="flex flex-wrap gap-4">
              <Switch>
                <Match when={state.provider() == "openai"}>
                  <ModelNameSelection
                    selected=""
                    options={[
                      "gpt-4-0125-preview",
                      "gpt-4-turbo-preview",
                      "gpt-4-1106-preview",
                      "gpt-4-vision-preview",
                      "gpt-4-1106-vision-preview",
                      "gpt-4",
                      "gpt-4-0613",
                      "gpt-4-32k",
                      "gpt-4-32k-0613",
                      "gpt-3.5-turbo-0125",
                      "gpt-3.5-turbo",
                      "gpt-3.5-turbo-1106",
                    ]}
                    onChange={(model) => {
                      setState("modelName", model);
                    }}
                  />
                </Match>
                <Match when={state.provider() == "anthropic"}>
                  <ModelNameSelection
                    selected=""
                    options={[
                      "claude-3-opus-20240229",
                      "claude-3-sonnet-20240229",
                      "claude-3-haiku-20240307",
                      "claude-2.1",
                      "claude-2.0",
                      "claude-instant-1.2",
                    ]}
                    onChange={(model) => {
                      console.log("model =", model);
                      setState("modelName", model);
                    }}
                  />
                </Match>
                <Match when={state.provider() == "groq"}>
                  <input
                    type="text"
                    name="modelName"
                    value={state.modelName()}
                    placeholder="Name of the model; eg 'mixtral-8x7b-32768'"
                    class="w-full px-2 py-1.5 text-sm border border-gray-200 bg2-gray-200 rounded outline-none focus:ring-1"
                    onInput={(e) => setState("modelName", e.target.value)}
                  />
                </Match>
              </Switch>
            </div>
          </div>
        </Show>
        <div class="text-xs text-center text-red-600 overflow-hidden text-ellipsis">
          <div class="line-clamp-3 whitespace-pre">{state.error() || " "}</div>
        </div>
        <div class="flex justify-end">
          <button
            class="px-4 py-1.5 rounded text-white bg-indigo-600 hover:bg-indigo-500"
            onClick={() => {
              setState("error", "");
              addModel.mutate({}).then(async (res) => {
                if (!res.ok) {
                  const error = await res.text();
                  setState("error", error);
                } else {
                  props.closeDialog();
                }
              });
            }}
          >
            Add Model
          </button>
        </div>
      </div>
    </Dialog>
  );
};

const TextModelProviders = (props: {
  value: string;
  onChange: (value: string) => void;
}) => {
  return (
    <>
      <RadioOption
        id="ollama"
        title="Ollama"
        name="provider"
        selected={props.value}
        onChange={props.onChange}
      />
      <RadioOption
        id="lmstudio"
        title="LM Studio"
        name="provider"
        selected={props.value}
        onChange={props.onChange}
      />
      <RadioOption
        id="openai"
        title="Open AI"
        name="provider"
        selected={props.value}
        onChange={props.onChange}
      />
      <RadioOption
        id="anthropic"
        title="Anthropic"
        name="provider"
        selected={props.value}
        onChange={props.onChange}
      />
      <RadioOption
        id="groq"
        title="Groq"
        name="provider"
        selected={props.value}
        onChange={props.onChange}
      />
    </>
  );
};

const ImageModelProviders = (props: {
  value: string;
  onChange: (value: string) => void;
}) => {
  return (
    <>
      <RadioOption
        id="playground"
        title="Playground"
        name="provider"
        selected={props.value}
        onChange={props.onChange}
      />
      <RadioOption
        id="stable-diffusion"
        title="Stable Diffusion"
        name="provider"
        selected={props.value}
        onChange={props.onChange}
      />
    </>
  );
};

const ModelNameSelection = (props: {
  selected: string;
  options: string[];
  onChange: (value: string) => void;
}) => {
  return (
    <For each={props.options}>
      {(option) => {
        return (
          <RadioOption
            id={option}
            title={option}
            name="modelName"
            selected={props.selected}
            onChange={props.onChange}
            class="w-56"
          />
        );
      }}
    </For>
  );
};

const RadioOption = (props: {
  id: string;
  title: string;
  name: string;
  selected: string;
  onChange: (value: string) => void;
  class?: string;
}) => {
  return (
    <label
      class="px-4 py-1.5 rounded space-y-1.5 bg-gray-50 text-gray-700 border border-gray-100 has-[:checked]:border-indigo-300 has-[:checked]:bg-indigo-100"
      classList={{
        [props.class!]: Boolean(props.class),
      }}
    >
      <div class="text-sm font-medium">{props.title}</div>
      <input
        name={props.name}
        type="radio"
        class="hidden"
        value={props.id}
        checked={props.id == props.selected}
        onChange={(e) => {
          console.log("e.target.value =", e.target.checked);
          if (e.target.checked) {
            console.log("props.value =", props.id);
            props.onChange(props.id);
          }
        }}
      />
    </label>
  );
};

const Input = (props: {
  name: string;
  placeholder: string;
  type: string;
  title: string;
  value: string;
  onInput: (value: string) => void;
}) => {
  return (
    <label class="space-y-1.5">
      <div class="text-base font-medium text-gray-800">{props.title}</div>
      <input
        type={props.type}
        name={props.name}
        value={props.value}
        placeholder={props.placeholder}
        class="w-full px-2 py-1.5 text-sm border border-gray-200 bg2-gray-200 rounded outline-none focus:ring-1"
        onInput={(e) => props.onInput(e.target.value)}
      />
    </label>
  );
};

export { AddModel };
