import { EditorView as CMEditorView, keymap } from "@codemirror/view";
import { basicSetup } from "codemirror";
import { indentWithTab } from "@codemirror/commands";
import { EditorState } from "@codemirror/state";
import { javascript } from "@codemirror/lang-javascript";
import { sql } from "@codemirror/lang-sql";

type CodeMirrorOptions = {
  lang: "javascript" | "sql";
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
        console.log(cm.state.doc.toString());
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

  Object.assign(editor, {
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
  });

  return editor as EditorView;
};

const CodeEditor = () => {
  return (
    <div
      class="code-editor"
      ref={(ele) =>
        attachEditor(ele, {
          lang: "javascript",
          value: "yooo",
        })
      }
    ></div>
  );
};

export { attachEditor, CodeEditor };
export type { EditorView };
