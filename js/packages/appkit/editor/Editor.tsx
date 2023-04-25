import {
  Show,
  For,
  batch,
  createContext,
  useContext,
  createMemo,
} from "solid-js";
import { createStore } from "@arena/solid-store";
import { klona } from "klona";
import type {
  BaseConfig,
  EditorContextProvider,
  EditorProps,
  EditorState,
  InternalEditor,
  Plugin,
} from "./types";
import { EditorStateContext } from "./state/withEditorState";

type EditorContext<T> = {} & Pick<
  EditorStateContext,
  "state" | "useWidgetById"
> &
  T;

const EditorContext = createContext<EditorContext<any>>();
function useEditorContext<T>() {
  return useContext<EditorContext<T>>(EditorContext)!;
}

function createEditorWithPlugins(): EditorContextProvider;

function createEditorWithPlugins<C, PS, Ctx>(
  plugin1: ReturnType<Plugin<C, PS, Ctx>>
): EditorContextProvider;

function createEditorWithPlugins<C1, PS1, Ctx1, C2, PS2, Ctx2>(
  plugin1: ReturnType<Plugin<C1, PS1, Ctx1>>,
  plugin2: ReturnType<Plugin<C2, PS2, Ctx2>>
): EditorContextProvider;

function createEditorWithPlugins<C1, PS1, Ctx1, C2, PS2, Ctx2, C3, PS3, Ctx3>(
  plugin1: ReturnType<Plugin<C1, PS1, Ctx1>>,
  plugin2: ReturnType<Plugin<C2, PS2, Ctx2>>,
  plugin3: ReturnType<Plugin<C3, PS3, Ctx3>>
): EditorContextProvider;

function createEditorWithPlugins<
  C1,
  PS1,
  Ctx1,
  C2,
  PS2,
  Ctx2,
  C3,
  PS3,
  Ctx3,
  C4,
  PS4,
  Ctx4
>(
  plugin1: ReturnType<Plugin<C1, PS1, Ctx1>>,
  plugin2: ReturnType<Plugin<C2, PS2, Ctx2>>,
  plugin3: ReturnType<Plugin<C3, PS3, Ctx3>>,
  plugin4: ReturnType<Plugin<C4, PS4, Ctx4>>
): EditorContextProvider;

function createEditorWithPlugins<Config, PluginState, Context>(
  ...plugins: ReturnType<Plugin<Config, PluginState, Context>>[]
) {
  return (props: EditorProps) => {
    const pluginResults: any = [];
    const editor = batch(() => {
      const internalEditor = createBaseEditor<PluginState, Context>(
        props.config
      );
      plugins.reduce((editor, plugin) => {
        const res = plugin(editor);
        pluginResults.push(res);
        return editor;
      }, internalEditor);

      return internalEditor;
    });

    const isReady = createMemo(() => {
      return pluginResults.reduce(
        (ready: boolean, p: any) => ready && (!p || p.isReady()),
        true
      );
    });

    return (
      <EditorContext.Provider value={editor.context}>
        <Show when={isReady()}>
          <For each={editor.components}>{(component) => <>{component}</>}</For>
          {props.children}
        </Show>
      </EditorContext.Provider>
    );
  };
}

function createBaseEditor<S, M>(config: BaseConfig) {
  config = klona(config);
  const [state, setState] = createStore<EditorState<any>>({
    _core: {
      config,
    },
    _plugins: {},
  });

  const editor = {
    context: {},
    components: [],
    state,
    setState,
    plugins: [],
  } as unknown as InternalEditor<S, M>;
  return editor;
}

export { createEditorWithPlugins, useEditorContext };
