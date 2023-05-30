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
      if (e.target != document.body) {
        return;
      }
      const compoundKey = `${e.ctrlKey ? "ctrl+" : ""}${
        e.shiftKey ? "shift+" : ""
      }${e.key.toLowerCase()}` as keyof typeof SHORTCUTS;
      SHORTCUTS[compoundKey]?.(context as unknown as CommandContext);
    };
    document.addEventListener("keydown", handleKeydown);
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
};

export { withKeyboardShortcuts };
