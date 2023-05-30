import { Accessor, createComputed, createMemo, untrack } from "solid-js";
import { Plugin } from "./types";
import { Widget } from "@arena/widgets/schema";
import { App } from "../../types";

type ComponentTreeNode = {
  /**
   * Widget id
   *
   * Root node is app, so id is null for that node
   */
  id: string | null;

  /**
   * Widget slug
   *
   * if the node is app instead of widget, app name is used
   */
  name: string;
  icon?: string;
  children: ComponentTreeNode[];
};

type Store = {
  childrenIds: Record<string, string[]>;
};

type ComponentTreeContext = {
  getComponentTree: Accessor<ComponentTreeNode | null>;

  /**
   * Pass `null` parentId to get root widgets
   *
   * returns the children of the given widget
   */
  useChildren: (parentId: string | null) => string[];
};

const withComponentTree: Plugin<
  void,
  { withComponentTree: Store },
  ComponentTreeContext
> =
  () =>
  ({ core, context, plugins }) => {
    const getComponentTree = createMemo(() => {
      const app = core.state.app();
      if (!app) {
        return null;
      }

      const prevChildrenByParentId = untrack(
        () => plugins.state.withComponentTree.childrenIds() || {}
      );
      return buildComponentTree(app, prevChildrenByParentId, plugins.setState);
    });

    createComputed(() => {
      // Note(sagar): call this here so that memo is tracked even when
      // getComponentTree isnt used from else where so that childrenIds
      // is computed
      void getComponentTree();
    });

    Object.assign(context, {
      getComponentTree,
      useChildren(parentId: string) {
        return plugins.state.withComponentTree.childrenIds[parentId]() || [];
      },
    });
  };

const buildComponentTree = (
  app: App,
  prevChildrenByParentId: any,
  setState: any
) => {
  const allWidgets = Object.values(app.widgets);
  const fistChildByParentId = new Map<string | null, string>();
  const siblingAfterWidgetById = new Map();
  allWidgets.forEach((w) => {
    const placeAfter = w.config.layout?.position?.after || null;
    const parentsFirstChild = fistChildByParentId.get(w.parentId);
    if (
      siblingAfterWidgetById.has(placeAfter) ||
      (placeAfter == null && parentsFirstChild && parentsFirstChild != w.id)
    ) {
      const widgetAfter = siblingAfterWidgetById.get(placeAfter);
      throw new Error(
        `More than 1 widgets [${widgetAfter},${w.id}] in a same position of parent: ${w.parentId}`
      );
    }
    if (placeAfter == null) {
      fistChildByParentId.set(w.parentId, w.id);
    } else {
      siblingAfterWidgetById.set(placeAfter, w.id);
    }
  });

  const { children } = sortChildrenAndUpdate(
    setState,
    app.widgets,
    prevChildrenByParentId,
    siblingAfterWidgetById,
    fistChildByParentId,
    null!
  );

  return {
    id: null,
    name: app.name,
    children,
  };
};

const sortChildrenAndUpdate = (
  setState: any,
  widgets: App["widgets"],
  prevChildrenByParentIds: Record<string, Widget["id"]>,
  siblingAfterWidgetById: Map<string, string>,
  fistChildByParentId: Map<string | null, string>,
  widgetId: Widget["id"]
): ComponentTreeNode => {
  const prevChildrenIds = prevChildrenByParentIds[widgetId] || [];
  const sortedChildrenIds = [];
  const firstChild = fistChildByParentId.get(widgetId);
  if (firstChild) {
    let i = 0;
    let childernChanged = false;
    let currentChild: string | undefined = firstChild;
    while (currentChild) {
      sortedChildrenIds.push(currentChild);
      if (currentChild != prevChildrenIds[i]) {
        childernChanged = true;
      }
      currentChild = siblingAfterWidgetById.get(currentChild);
      i += 1;
    }
    if (childernChanged) {
      setState("withComponentTree", "childrenIds", widgetId, sortedChildrenIds);
    }
  }

  if (prevChildrenIds.length != sortedChildrenIds.length) {
    // reset the children ids if the number of children changed
    setState("withComponentTree", "childrenIds", widgetId, sortedChildrenIds);
  }
  return {
    id: widgetId,
    name: widgets[widgetId]?.slug || null!,
    children: sortedChildrenIds.map((childId) =>
      sortChildrenAndUpdate(
        setState,
        widgets,
        prevChildrenByParentIds,
        siblingAfterWidgetById,
        fistChildByParentId,
        childId
      )
    ),
  };
};

export { withComponentTree };
export type { ComponentTreeNode, ComponentTreeContext };
