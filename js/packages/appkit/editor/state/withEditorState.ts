import { createResource, batch, createComputed, Accessor } from "solid-js";
import { Store } from "@arena/solid-store";
import { uniqueId } from "@arena/uikit";
import { App } from "../../App";
import { Plugin } from "../types";
import { Widget, Template } from "../../widget";
import { useApiContext } from "../../ApiContext";

type EditorStateContext = {
  useWidgetById: (id: string) => Store<Widget>;
  addWidget: (
    parentId: string,
    templateId: string,
    templateMetadata: Template.Metadata<any>
  ) => Promise<Widget>;

  /**
   * Keeps track of widgetId -> html node
   * If a widget node is removed, this should be called with node = null
   */
  registerWidgetNode: (widgetId: string, node: HTMLElement | null) => void;
  /**
   * Returns the HTML node of a rendered widget
   */
  useWidgetNode: (widgetId: string) => HTMLElement | null;
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

    const addWidget: EditorStateContext["addWidget"] = async (
      parentId,
      templateId,
      templateMetadata
    ) => {
      const defaultDataConfig = Object.fromEntries(
        Object.entries(templateMetadata.data).map(([field, { dataSource }]) => {
          const { type, default: config } = dataSource;
          return [field, { type, config }];
        })
      ) as Widget["config"]["data"];

      const newWidget = {
        id: uniqueId(),
        name: templateMetadata.name,
        // TODO(sagar): generate slug
        slug: "",
        description: templateMetadata.description || "",
        parentId,
        appId: config.appId,
        templateId,
        config: {
          // TODO(sagar): set widget's config in server
          data: defaultDataConfig,
          class: templateMetadata.class,
        },
      };

      const w = await api.routes.addWidget(newWidget);
      core.setState("app", "widgets", (widgets) => {
        return [...widgets, w];
      });
      return w;
    };

    Object.assign(context, {
      useWidgetById(id: string) {
        return core.state.widgetsById[id];
      },
      addWidget,
      registerWidgetNode(widgetId, node) {
        plugins.setState("withEditorState", "widgetNodes", widgetId, node);
      },
      useWidgetNode(widgetId) {
        return plugins.state.withEditorState.widgetNodes()[widgetId];
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

export { withEditorState };
export type { EditorStateConfig, EditorStateContext };
