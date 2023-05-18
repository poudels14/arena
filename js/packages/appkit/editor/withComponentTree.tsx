import { Accessor, createMemo } from "solid-js";
import { Plugin } from "./types";
import { Widget } from "../widget/types";

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
  childrenMap: Record<string, string[]>;
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
      const widgets = app.widgets;
      const map = {};
      plugins.setState("withComponentTree", "childrenMap", map);
      return {
        id: null,
        slug: null,
        title: app.name,
        children: getChildren(widgets, map, null),
      };
    });

    Object.assign(context, {
      getComponentTree,
      useChildren(parentId: string) {
        return plugins.state.withComponentTree.childrenMap[parentId]() || [];
      },
    });
  };

const getChildren = (
  widgets: Widget[],
  map: Record<string, string[]>,
  parentId: string | null
): ComponentTreeNode[] => {
  return widgets
    .filter((w) => w.parentId === parentId)
    .map((w) => {
      if (!map[parentId!]) {
        map[parentId!] = [];
      }
      map[parentId!].push(w.id);
      const children = getChildren(widgets, map, w.id);
      return {
        id: w.id,
        slug: w.slug,
        title: w.name,
        children,
      };
    });
};

export { withComponentTree };
export type { ComponentTreeNode, ComponentTreeContext };
