import { EditorView as CMEditorView, keymap } from "@codemirror/view";
import { basicSetup } from "codemirror";
import { indentWithTab } from "@codemirror/commands";
import { EditorState } from "@codemirror/state";
import { javascript } from "@codemirror/lang-javascript";
import { sql } from "@codemirror/lang-sql";
import { createEffect, createMemo, untrack } from "solid-js";

type CodeMirrorOptions = {
  lang: "javascript" | "sql" | "text";
  value: string;
  theme?: "lucario";
  onChange?: (v: string) => void;
};

type EditorView = CMEditorView & {
  getValue: () => string;
  setValue: (value: string) => void;
};

const attachEditor = (parent: Element, options: CodeMirrorOptions) => {
  const { lang } = options;
  const extensions = [
    basicSetup,
    keymap.of([indentWithTab]),
    CMEditorView.updateListener.of((cm) => {
      if (cm.docChanged) {
        options.onChange?.(cm.state.doc.toString());
      }
    }),
  ];

  switch (lang) {
    case "javascript":
      extensions.push(javascript());
      break;
    case "sql":
      extensions.push(sql());
      break;
  }

  const editor = new CMEditorView({
    state: EditorState.create({
      doc: options.value,
      extensions,
    }),
    parent,
  });

  return Object.assign(editor, {
    getValue() {
      return editor.state.doc.toString();
    },
    setValue(value: string) {
      editor.dispatch({
        changes: {
          from: 0,
          to: editor.state.doc.length,
          insert: value,
        },
      });
    },
  }) as EditorView;
};

const CodeEditor = (props: CodeMirrorOptions) => {
  let ref: any;

  const lang = createMemo(() => props.lang);
  const value = createMemo(() => props.value);

  createEffect((prev?: EditorView) => {
    // Note(sagar): create new editor instance if lang changed, else just update the value
    void lang();
    prev && prev.destroy();
    const editor = untrack(() => attachEditor(ref, props));
    createEffect(() => {
      const v = value();
      editor && v != editor.getValue() && editor.setValue(v);
    });
    return editor;
  });
  return <div class="code-editor" ref={ref}></div>;
};

export { attachEditor, CodeEditor };
export type { EditorView };
