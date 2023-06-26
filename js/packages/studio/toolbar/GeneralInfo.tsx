import { useEditorContext } from "../editor";
import { createMemo } from "solid-js";

const GeneralInfo = () => {
  const { getSelectedWidgets, useWidgetById } = useEditorContext();

  const widget = createMemo(() => {
    const activeWidgets = getSelectedWidgets();
    const widgetId = activeWidgets[0];
    return useWidgetById(widgetId)();
  });

  return (
    <div class="flex flex-col w-full h-full px-4">
      <div class="flex-1 flex flex-col text-sm text-white space-y-2">
        <div class="flex py-2 space-x-3 bg-brand-12/70">
          <div class="w-40 px-2">Name</div>
          <div>{widget().name}</div>
        </div>
        <div class="flex py-2 space-x-3 bg-brand-12/70">
          <div class="w-40 px-2">Id</div>
          <div>{widget().id}</div>
        </div>
      </div>
    </div>
  );
};

export default GeneralInfo;
