import { children } from "solid-js";

type WidgetProps<State> = {
  state: State;
  setState: any;
  /**
   * Whether ther widget was terminated
   */
  terminated: boolean;
  /**
   * Terminates live widgets like timer, workflow, etc
   */
  terminate: () => void;
};

const WigetContainer = (props: {
  Widget: any;
  metadata: any;
  state: any;
  UI: { Markdown: any; Table: any };
}) => {
  const widget = children(() =>
    props.Widget({
      metadata: props.metadata,
      state: props.state,
      UI: props.UI,
    })
  );
  return (
    <div class="widget-container px-4 bg-gray-50">
      <div class="">{widget()}</div>
    </div>
  );
};

export { WigetContainer };
export type { WidgetProps };
