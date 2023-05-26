import { createResource, batch, createComputed, Accessor } from "solid-js";
import { Store, StoreSetter } from "@arena/solid-store";
import { uniqueId } from "@arena/uikit";
import cleanSet from "clean-set";
import { App } from "../types/app";
import { Plugin } from "./plugins/types";
import { Widget } from "@arena/widgets/schema";
import { useApiContext } from "../ApiContext";
import { MutationResponse } from "../types";
import { AnyInternalEditor } from "./plugins/types";

type EditorStateContext = {
  useWidgetById: (id: string) => Store<Widget>;
  addWidget: (props: {
    templateId: string;
    parentId: string;
    position: {
      /**
       * Id of the widget to place the new widget after
       */
      after: string | null;
      /**
       * Id of the widget that existsin the location of this widget
       * This is used to properly re-order the widgets
       */
      before: string | null;
    };
  }) => Promise<Widget>;
  /**
   * This is a promise
   */
  updateWidget: StoreSetter<Record<string, Omit<Widget, "id" | "template">>>;

  /**
   * Keeps track of widgetId -> html node
   * If a widget node is removed, this should be called with node = null
   */
  registerWidgetNode: (widgetId: string, node: HTMLElement | null) => void;
  /**
   * Returns the HTML node of a rendered widget
   */
  useWidgetNode: (widgetId: string) => Accessor<HTMLElement | null>;
  /**
   * Highlight a widget with the given id; if replace = true, sets the selected
   * widget to just the given widget id, else adds to existing list of selected
   * widgets.
   *
   * replace = true by default
   */
  setSelectedWidget: (widgetId: string, replace?: boolean) => void;
  getSelectedWidgets: Accessor<string[]>;
};

type EditorState = {
  ready: boolean;
  selectedWidgets: string[];
  widgetNodes: Record<string, HTMLElement | null>;
};

type EditorStateConfig = {
  appId: string;
};

const withEditorState: Plugin<
  EditorStateConfig,
  { withEditorState: EditorState },
  EditorStateContext
> =
  (config) =>
  ({ core, plugins, context }) => {
    const api = useApiContext();
    const [getApp] = createResource(
      () => config.appId,
      async (appId) => {
        return await api.routes.fetchApp(appId);
      }
    );

    plugins.setState("withEditorState", {
      ready: false,
      selectedWidgets: [],
      widgetNodes: {},
    });

    createComputed(() => {
      const app = getApp() as App;
      batch(() => {
        core.setState("app", app);
      });
    });

    createComputed(() => {
      const app = core.state.app();
      if (app) {
        const widgetsById = Object.fromEntries(
          app.widgets.map((widget) => {
            return [widget.id, widget];
          })
        );
        core.setState("widgetsById", widgetsById);
        plugins.setState("withEditorState", "ready", true);
      }
    });

    const addWidget: EditorStateContext["addWidget"] = async ({
      templateId,
      parentId,
      position,
    }) => {
      const newWidget = {
        id: uniqueId(),
        appId: config.appId,
        templateId,
        parentId,
        position,
      };

      const updates = await api.routes.addWidget(newWidget);
      updateState(core, updates);
      return updates?.widgets?.created?.[0]!;
    };

    const updateWidget: EditorStateContext["updateWidget"] = async (
      ...path: any
    ) => {
      const widgetId = path.shift();
      const value = path.pop();
      const updates = await api.routes.updateWidget({
        id: widgetId,
        ...cleanSet({}, path, value),
      });
      // TODO(sp): revert changed if API call failed
      updateState(core, updates);
    };

    Object.assign(context, {
      useWidgetById(id: string) {
        return core.state.widgetsById[id];
      },
      addWidget,
      updateWidget,
      registerWidgetNode(widgetId, node) {
        plugins.setState("withEditorState", "widgetNodes", widgetId, node);
      },
      useWidgetNode(widgetId) {
        return plugins.state.withEditorState.widgetNodes[widgetId];
      },
      setSelectedWidget(widgetId, replace = true) {
        plugins.setState("withEditorState", "selectedWidgets", (widgets) => {
          return replace ? [widgetId] : [...widgets, widgetId];
        });
      },
      getSelectedWidgets() {
        return plugins.state.withEditorState.selectedWidgets();
      },
    } as EditorStateContext);

    return {
      isReady() {
        return plugins.state.withEditorState.ready();
      },
    };
  };

const updateState = (
  core: AnyInternalEditor["core"],
  updates: MutationResponse
) => {
  const { created: createdWidgets = [], updated: updatedWidgets = [] } =
    updates.widgets || {};
  core.setState("app", "widgets", (widgets) => {
    widgets = widgets.map((w) => {
      return updatedWidgets.find((u) => u.id == w.id) ?? w;
    });
    createdWidgets.forEach((w) => widgets.push(w));
    return widgets;
  });
};

export { withEditorState };
export type { EditorStateConfig, EditorStateContext };
