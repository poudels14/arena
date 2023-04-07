import { Accessor, batch, onCleanup, splitProps, untrack } from "solid-js";
import { Plugin } from "..";
import { Header } from "../column";
import { InternalTable } from "../types";

type ResizableColumnsConfig = {};

type State = {
  resizableColumns: {
    columnWidths: Record<string, number>;
    resizing: {
      headerId: string;
      startX: number;
      currentWidth: number;
      endX?: number;
    };
  };
};

type Methods = {};

const createResizer =
  (table: InternalTable<State, Methods>) =>
  (resizer: Node, value: Accessor<[header: Header, th: any]>) => {
    const [header, th] = value();
    const updateWidth = (resizing: any, endX: number) => {
      const newWidth = resizing.currentWidth + endX - resizing.startX;
      th.style.width = newWidth + "px";
      return newWidth;
    };

    let stopResizing: any;
    stopResizing = (e: any) =>
      untrack(() => {
        window.removeEventListener("pointermove", resize);
        window.removeEventListener("pointerup", stopResizing);
        document.body.style.userSelect = "auto";

        const resizing = table.state._plugins.resizableColumns.resizing();
        const newWidth = updateWidth(resizing, e.clientX);
        batch(() => {
          table.setState(
            "_plugins",
            "resizableColumns",
            "columnWidths",
            resizing.headerId,
            newWidth
          );
          table.setState(
            "_plugins",
            "resizableColumns",
            "resizing",
            undefined!
          );
        });
      });

    const startResizing = (e: any) => {
      window.addEventListener("pointerup", stopResizing);
      window.addEventListener("pointermove", resize);
      document.body.style.userSelect = "none";

      table.setState("_plugins", "resizableColumns", "resizing", {
        headerId: header.id,
        currentWidth: th.offsetWidth,
        startX: e.clientX,
      });
    };

    const resize = (e: any) =>
      untrack(() => {
        const resizing = table.state._plugins.resizableColumns.resizing();
        updateWidth(resizing, e.clientX);

        table.setState(
          "_plugins",
          "resizableColumns",
          "resizing",
          "endX",
          e.clientX
        );
      });

    resizer.addEventListener("pointerdown", startResizing);
    onCleanup(() => {
      resizer.removeEventListener("pointerdown", startResizing);
    });
  };

const withResizableColumns: Plugin<ResizableColumnsConfig, State, Methods> = (
  config
) => {
  return (table) => {
    table.setState("_plugins", "resizableColumns", {
      columnWidths: {},
    });

    const resizer = createResizer(table);
    let thRef: any = null;
    Object.assign(table.ui, {
      Th: (props: Parameters<typeof table.ui.Th>[0]) => {
        const [_, rest] = splitProps(props, [
          "header",
          "class",
          "classList",
          "children",
        ]);
        return (
          <th
            {...rest}
            classList={{
              [props.class!]: Boolean(props.class),
              "p-0 m-0 relative after:(absolute,left-0,bottom-0,w-full,h-[1px],shadow-lg,shadow-gray-300)":
                true,
              ...(props.classList ?? {}),
            }}
            ref={thRef}
          >
            <div class="flex">
              <div class="flex-1">{props.children}</div>
              <div
                class="resizer w-[4px] cursor-ew-resize"
                draggable={false}
                // @ts-ignore
                use:resizer={[props.header, thRef]}
              ></div>
            </div>
          </th>
        );
      },
    });
  };
};

export { withResizableColumns };
