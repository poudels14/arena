import { Template } from "@arena/widgets";
import { InternalEditor, Plugin } from "./types";
import { Slot } from "../../Slot";
import { EditorStateContext } from "../withEditorState";

type WidgetPropsContext = {
  Editor: {
    Slot: typeof Slot;
    useContext: Template.Editor["useContext"];
  };
};

const withWidgetProps: Plugin<void, void, WidgetPropsContext> = () => {
  return (editor) => {
    const ctx = (editor as unknown as InternalEditor<any, EditorStateContext>)
      .context;

    const useContext = () => {
      return {
        useResources: ctx.useResources,
      };
    };

    Object.assign(editor.context, {
      Editor: {
        Slot,
        useContext,
      },
    });
  };
};

export { withWidgetProps };
export type { WidgetPropsContext };
