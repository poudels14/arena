import { onCleanup } from "solid-js";
import { InternalEditor, Plugin } from "./types";

const withKeyboardShortcuts: Plugin<{}, void, void> = (config) => {
  return ({ context }) => {
    const handleKeydown = (e: KeyboardEvent) => {
      if (e.target != document.body) {
        return;
      }
      const compoundKey = `${e.ctrlKey ? "ctrl+" : ""}${
        e.shiftKey ? "shift+" : ""
      }${e.key.toLowerCase()}` as keyof typeof SHORTCUTS;
      SHORTCUTS[compoundKey]?.(context);
    };
    document.addEventListener("keydown", handleKeydown);
    onCleanup(() => document.removeEventListener("keydown", handleKeydown));

    Object.assign(context, {});
  };
};

const SHORTCUTS = {
  escape: (context: InternalEditor<any, any>["context"]) => {
    context.setSelectedWidgets([], true);
  },
};

export { withKeyboardShortcuts };
