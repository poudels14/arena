import { onCleanup } from "solid-js";
import { InternalEditor, Plugin } from "./types";
import { EditorStateContext } from "../withEditorState";
import { ComponentTreeContext } from "./withComponentTree";

type CommandContext = InternalEditor<
  any,
  EditorStateContext & ComponentTreeContext
>["context"];
type ShortcutCommand = (context: CommandContext) => void;

const withKeyboardShortcuts: Plugin<{}, void, void> = (config) => {
  return ({ context }) => {
    const handleKeydown = (e: KeyboardEvent) => {
      let key = e.code;
      key = key == " " ? "space" : key;
      const compoundKey = `${e.ctrlKey ? "ctrl+" : ""}${
        e.shiftKey ? "shift+" : ""
      }${key.toLowerCase()}` as keyof typeof SHORTCUTS;

      let shortcut;
      if ((shortcut = SHORTCUTS[compoundKey])) {
        e.preventDefault();
        e.stopPropagation();
        shortcut(context as unknown as CommandContext);
      }
    };
    document.addEventListener("keydown", handleKeydown, {
      capture: true,
    });
    onCleanup(() => document.removeEventListener("keydown", handleKeydown));

    Object.assign(context, {});
  };
};

const SHORTCUTS: Record<string, ShortcutCommand> = {
  escape: (context: CommandContext) => {
    context.setSelectedWidgets([], true);
  },
  delete: (context: CommandContext) => {
    const [selectedWidgetId] = context.getSelectedWidgets();
    if (selectedWidgetId) {
      const widget = context.useWidgetById(selectedWidgetId)();
      const children = context.useChildren(widget.parentId);
      const widgetIndex = children.findIndex((c) => c == widget.id);
      let before = null;
      if (widgetIndex < children.length - 1) {
        before = children[widgetIndex + 1];
      }

      context.deleteWidget({
        id: selectedWidgetId,
        config: {
          layout: {
            position: {
              after: null,
              before,
            },
          },
        },
      });
    }
  },
  "ctrl+space": (context) => {
    context.setViewOnly(!context.isViewOnly());
  },
};

export { withKeyboardShortcuts };
