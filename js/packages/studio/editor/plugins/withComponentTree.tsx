import { Accessor, createComputed, createMemo, untrack } from "solid-js";
import { Plugin } from "./types";
import { Widget } from "@arena/widgets/schema";

type ComponentTreeNode = {
  /**
   * Widget id
   * Root node is app, so id is null for that node
   */
  id: string | null;

  /**
   * Widget slug
   * null if the node is app instead of widget
   */
  slug: string | null;
  title: string;
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

      const childrenIds = {};
      const children = collectChildren(app.widgets, childrenIds, null);
      untrack(() => {
        plugins.setState("withComponentTree", "childrenIds", childrenIds);
      });

      return {
        id: null,
        slug: null,
        title: app.name,
        children,
      };
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

const collectChildren = (
  widgets: Widget[],
  childrenIds: Record<string, string[]>,
  parentId: string | null
) => {
  const placeAfterWidget = new Map();
  const children = new Map();
  widgets
    .filter((w) => w.parentId === parentId)
    .forEach((w) => {
      const placeAfter = w.config.layout?.position?.after || null;
      if (placeAfterWidget.get(placeAfter)) {
        throw new Error(
          "More than 1 widget in a same position of parent: " + parentId
        );
      }
      placeAfterWidget.set(placeAfter, w.id);
      children.set(w.id, {
        id: w.id,
        slug: w.slug,
        title: w.name,
        children: collectChildren(widgets, childrenIds, w.id),
      });
    });

  const sortedChildrenNode = [];
  const sortedChildrenIds = [];
  let nextChild = null;
  while ((nextChild = placeAfterWidget.get(nextChild))) {
    const nextNode = children.get(nextChild);
    sortedChildrenNode.push(nextNode);
    sortedChildrenIds.push(nextNode.id);
  }

  childrenIds[parentId!] = sortedChildrenIds;
  return sortedChildrenNode;
};

export { withComponentTree };
export type { ComponentTreeNode, ComponentTreeContext };
