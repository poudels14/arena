import { createResource, createEffect, batch, createComputed } from "solid-js";
import { Store } from "@arena/solid-store";
import { uniqueId } from "@arena/uikit";
import { App } from "../../App";
import { Plugin } from "../types";
import { Widget, Template } from "../../widget";
import { widgetSchema } from "../../widget/types";

type EditorStateContext = {
  useWidgetById: (id: string) => Store<Widget>;
  addWidget: (
    parentId: string,
    templateId: string,
    templateMetadata: Template.Metadata<any>
  ) => Promise<Widget>;
};

type EditorState = {
  ready: boolean;
};

type EditorStateConfig = {
  appId: string;
  fetchApp: (appId: string) => Promise<App>;
  addWidget: (widget: {
    id: string;
    appId: string;
    name: string;
    slug: string;
    description?: string;
    parentId: string | null;
    templateId: string;
    config: Widget["config"];
  }) => Promise<Widget>;
  updateWidget: (widget: any) => Promise<Widget>;
};

const withEditorState: Plugin<
  EditorStateConfig,
  { withEditorState: EditorState },
  EditorStateContext
> =
  (config) =>
  ({ core, plugins, context }) => {
    const [getApp] = createResource(
      () => config.appId,
      async (appId) => {
        return await config.fetchApp(appId);
      }
    );

    plugins.setState("withEditorState", { ready: false });

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
          data: defaultDataConfig,
        },
      };

      const w = await config.addWidget(newWidget);
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
    } as EditorStateContext);

    return {
      isReady() {
        return plugins.state.withEditorState.ready();
      },
    };
  };

export { withEditorState };
export type { EditorStateConfig, EditorStateContext };
