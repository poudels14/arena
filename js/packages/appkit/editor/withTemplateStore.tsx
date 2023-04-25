import { JSX } from "solid-js";
import { Plugin } from "./types";
import { Store, StoreSetter, createStore } from "@arena/solid-store";
import { Widget } from "../widget/types";
import { TemplateMetadata } from "../widget";

type Templates = Record<
  string,
  {
    metadata: TemplateMetadata<any>;
    Component: (props: any) => JSX.Element;
  }
>;

type TemplateStore = {
  templates: Templates;
};

type TemplateStoreContext = {
  useTemplates: () => Store<Templates>;
  useTemplate: (template: Widget["template"]["id"]) => {
    metadata: TemplateMetadata<any>;
    Component: (props: any) => JSX.Element;
  };
  setTemplatesStore: StoreSetter<TemplateStore>;
};

/**
 * Note(sagar): this isn't reactive; meaning, if {@param template}
 * changes, the return value doesn't change
 */
const useTemplateBuilder =
  (templatesStore: any) => (templateId: Widget["template"]["id"]) => {
    const templates = templatesStore.templates();

    return {
      metadata: templates[templateId].metadata,
      Component: templates[templateId].Component,
    };
  };

const withTemplateStore: Plugin<{ templates: Templates }, {}, {}> =
  (config) => (editor) => {
    const [store, setStore] = createStore({
      templates: config.templates,
    });

    const useTemplate = useTemplateBuilder(store);

    Object.assign(editor.context, {
      useTemplates: () => store.templates,
      setTemplatesStore: setStore,
      useTemplate,
    });
  };

export { withTemplateStore };
export type { TemplateStoreContext };
