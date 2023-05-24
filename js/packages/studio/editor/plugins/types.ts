import { Store, StoreSetter } from "@arena/solid-store";
import { JSX } from "solid-js";
import { App } from "../../types/app";
import { Widget } from "@arena/widgets/schema";

type CoreState = {
  app: App;
  widgetsById: Record<string, Widget>;
};

type BaseConfig = {};

type InternalEditor<PluginsState, Context> = {
  context: Context & {
    state: Store<CoreState>;
  };

  /**
   * These are components added by plugins
   */
  components: (() => JSX.Element)[];

  core: {
    config: BaseConfig;
    state: Store<{
      app: App;
      widgetsById: Record<string, Widget>;
    }>;
    setState: StoreSetter<CoreState>;
  };

  plugins: {
    state: Store<PluginsState>;
    setState: StoreSetter<PluginsState>;
  };
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
  CoreState,
  AnyInternalEditor,
  EditorContextProvider,
  EditorProps,
  Plugin,
};
