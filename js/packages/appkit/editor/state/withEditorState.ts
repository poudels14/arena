import { createResource, createEffect, batch } from "solid-js";
import { Store } from "@arena/solid-store";
import { App } from "../../App";
import { Widget } from "../../widget/types";
import { Plugin } from "../types";

type EditorStateContext = {
  state: Store<Pick<EditorState, "app">>;
  useWidgetById: (id: string) => Store<Widget>;
  isAppStateReady: () => boolean;
};

type EditorState = {
  app: App;
  widgetsById: Record<string, Widget>;
  ready: boolean;
};

type EditorStateConfig = {
  appId: string;
  fetchApp: (appId: string) => Promise<App>;
};

const withEditorState: Plugin<
  EditorStateConfig,
  { withEditorState: EditorState },
  {}
> = (config) => (editor) => {
  const [getApp] = createResource(
    () => config.appId,
    async (appId) => {
      return await config.fetchApp(appId);
    }
  );

  editor.setState("_plugins", "withEditorState", {
    app: undefined,
    ready: false,
  });

  createEffect(() => {
    const app = getApp() as App;
    batch(() => {
      editor.setState("_plugins", "withEditorState", "app", app);
    });
  });

  createEffect(() => {
    const app = editor.state._plugins.withEditorState.app();
    if (app) {
      const widgetsById = Object.fromEntries(
        app.widgets.map((widget) => {
          return [widget.id, widget];
        })
      );
      editor.setState(
        "_plugins",
        "withEditorState",
        "widgetsById",
        widgetsById
      );
      editor.setState("_plugins", "withEditorState", "ready", true);
    }
  });

  const useWidgetById = (id: string) => {
    return editor.state._plugins.withEditorState.widgetsById[id];
  };

  Object.assign(editor.context, {
    state: editor.state._plugins.withEditorState,
    useWidgetById,
  });

  return {
    isReady() {
      return editor.state._plugins.withEditorState.ready();
    },
  };
};

export { withEditorState };
export type { EditorStateContext };
