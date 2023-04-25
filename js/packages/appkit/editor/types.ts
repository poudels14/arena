import { Store, StoreSetter } from "@arena/solid-store";
import { JSX } from "solid-js";

type EditorState<PluginsState> = {
  _core: {
    config: BaseConfig;
  };

  /**
   * Internal state stored by plugins using plugin name as a key
   */
  _plugins: PluginsState;
};

type BaseConfig = {};

type InternalEditor<PluginsState, Context> = {
  context: Context;

  /**
   * These are components added by plugins
   */
  components: (() => JSX.Element)[];

  /**
   * Current state of the table
   */
  state: Store<EditorState<PluginsState>>;

  setState: StoreSetter<EditorState<PluginsState>>;
};

type AnyInternalEditor = InternalEditor<unknown, unknown>;

type EditorProps = {
  config?: any;
  children: JSX.Element;
};

type EditorContextProvider = (props: EditorProps) => JSX.Element;

type Plugin<PluginConfig, PluginsState, Context> = (config: PluginConfig) => (
  table: InternalEditor<PluginsState, Context>
) => void | {
  isReady?: () => boolean;
};

export type {
  BaseConfig,
  InternalEditor,
  EditorState,
  AnyInternalEditor,
  EditorContextProvider,
  EditorProps,
  Plugin,
};
