import {
  createResource,
  batch,
  createComputed,
  Accessor,
  untrack,
  createSelector,
  createMemo,
} from "solid-js";
import { Store, StoreSetter, createSyncedStore } from "@arena/solid-store";
import isEqual from "fast-deep-equal/es6";
// @ts-expect-error
import delve from "dlv";
import { uniqueId } from "@arena/sdk/utils/uniqueId";
import cleanSet from "clean-set";
import { App, Resource } from "../types/app";
import { Plugin } from "./plugins/types";
import { Widget } from "@arena/widgets/schema";
import { useApiContext } from "../ApiContext";
import type { ApiRoutes } from "../ApiContext";
import { MutationResponse } from "../types";
import { AnyInternalEditor } from "./plugins/types";

type EditorStateContext = {
  isEditable: Accessor<boolean>;
  isViewOnly: Accessor<boolean>;
  setViewOnly: (viewOnly: boolean) => void;
  useWidgetById: (id: string) => Store<Widget>;
  addWidget: (
    props: Omit<Parameters<ApiRoutes["addWidget"]>[0], "id" | "appId">
  ) => Promise<Widget>;
  /**
   * This is a promise
   */
  updateWidget: StoreSetter<Record<string, Omit<Widget, "id" | "template">>>;

  /**
   * Delete the widget corresponding to the given id
   */
  deleteWidget: (req: Parameters<ApiRoutes["deleteWidget"]>[0]) => void;

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
  setSelectedWidgets: (ids: Widget["id"][], replace?: boolean) => void;
  getSelectedWidgets: Accessor<string[]>;
  isWidgetSelected: (id: Widget["id"]) => boolean;

  useResources: (filter?: { type?: string }) => Resource[];
};

type EditorState = {
  selectedWidgets: string[];
  widgetNodes: Record<string, HTMLElement | null>;
};

type EditorStateConfig = {
  appId: string;
  viewOnly?: boolean;
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

    const [syncedStore, setSyncedStored] = createSyncedStore<{
      viewOnly: boolean;
    }>(
      {
        viewOnly: config.viewOnly || false,
      },
      {
        storeKey: "studio/withEditorState",
      }
    );

    plugins.setState("withEditorState", {
      selectedWidgets: [],
      widgetNodes: {},
    });
    const untrackedViewOnly = untrack(() => syncedStore.viewOnly);

    const getSelectedWidgets = createMemo(
      () => {
        const widgets = core.state.app.widgets();
        if (!widgets) {
          return [];
        }
        const selectedWidgets = plugins.state.withEditorState.selectedWidgets();
        const allIds = Object.keys(widgets);
        return selectedWidgets.filter((w) => allIds.includes(w));
      },
      [],
      {
        equals(prev, next) {
          return (
            prev.length == next.length &&
            next.reduce((eq, n, i) => eq && n == prev[i], true)
          );
        },
      }
    );

    const isWidgetSelected = createSelector(
      getSelectedWidgets,
      (id: string, selected) => !syncedStore.viewOnly() && selected.includes(id)
    );

    createComputed(() => {
      const app = getApp() as App;
      batch(() => {
        core.setState("app", app);
      });
    });

    const addWidget: EditorStateContext["addWidget"] = async ({
      templateId,
      parentId,
      config: widgetConfig,
    }) => {
      const newWidget = {
        id: uniqueId(),
        appId: config.appId,
        templateId,
        parentId,
        config: widgetConfig,
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
      const payload = cleanSet({}, path, value);
      const widget = untrack(core.state.app.widgets[widgetId]);
      if (
        untrackedViewOnly() ||
        isEqual(delve(payload, path), delve(widget, path))
      ) {
        return;
      }
      const updates = await api.routes.updateWidget({
        id: widgetId,
        ...payload,
      });
      // TODO(sp): revert changed if API call failed

      // update the state in a transition so that if widget data is reloaded,
      // it doesn't trigger the suspense fallback
      updateState(core, updates);
    };

    const deleteWidget: EditorStateContext["deleteWidget"] = async (req) => {
      const updates = await api.routes.deleteWidget(req);
      updateState(core, updates);
    };

    Object.assign(context, {
      isEditable() {
        let editable = core.state.app.template.editor!.editable!();
        return editable == undefined ? true : editable;
      },
      isViewOnly() {
        return syncedStore.viewOnly();
      },
      setViewOnly(viewOnly) {
        setSyncedStored("viewOnly", viewOnly);
      },
      useWidgetById(id: string) {
        return core.state.app.widgets[id];
      },
      addWidget,
      updateWidget,
      deleteWidget,
      registerWidgetNode(widgetId, node) {
        plugins.setState("withEditorState", "widgetNodes", widgetId, node);
      },
      useWidgetNode(widgetId) {
        return plugins.state.withEditorState.widgetNodes[widgetId];
      },
      setSelectedWidgets(widgetIds, replace = true) {
        if (untrackedViewOnly()) {
          return;
        }
        plugins.setState("withEditorState", "selectedWidgets", (widgets) => {
          return replace ? widgetIds : [...widgets, ...widgetIds];
        });
      },
      getSelectedWidgets,
      isWidgetSelected,
      useResources(filter) {
        return Object.values(core.state.app.resources()).filter((r) => {
          return !filter?.type || filter?.type == r.type;
        });
      },
    } as EditorStateContext);

    return {
      isReady() {
        return Boolean(core.state.app());
      },
    };
  };

const updateState = (
  core: AnyInternalEditor["core"],
  updates: MutationResponse
) => {
  const {
    created: createdWidgets = [],
    updated: updatedWidgets = [],
    deleted: deletedWidgets = [],
  } = updates.widgets || {};

  batch(() => {
    updatedWidgets.forEach((w) => core.setState("app", "widgets", w.id, w));
    createdWidgets.forEach((w) => core.setState("app", "widgets", w.id, w));
    deletedWidgets.forEach((w) =>
      core.setState("app", "widgets", w.id, undefined!)
    );
  });
};

export { withEditorState };
export type { EditorStateConfig, EditorStateContext };
