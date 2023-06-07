import {
  Accessor,
  batch,
  onCleanup,
  onMount,
  splitProps,
  untrack,
} from "solid-js";
import { Plugin } from "..";
import { Header } from "../column";
import { InternalTable } from "../types";

type ResizableColumnsConfig = {
  columnWidths?: Record<string, number>;
};

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

type ResizableColumnsMethods = {
  getColumnWidths: Accessor<Record<string, number>>;
};

const createResizer =
  (table: InternalTable<State, ResizableColumnsConfig>) =>
  (resizer: Node, value: Accessor<[header: Header, th: any]>) => {
    const [header, th] = value();
    const updateWidth = (resizing: any, endX: number) => {
      const newWidth = Math.round(
        resizing.currentWidth + endX - resizing.startX
      );
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

const withResizableColumns: Plugin<
  ResizableColumnsConfig,
  State,
  ResizableColumnsMethods
> = (config) => {
  return (table) => {
    table.setState("_plugins", "resizableColumns", {
      columnWidths: config.columnWidths || {},
    });

    Object.assign(table, {
      getColumnWidths() {
        return table.state._plugins.resizableColumns.columnWidths();
      },
    });

    const resizer = createResizer(table);
    Object.assign(table.Ui, {
      Th: (props: Parameters<typeof table.Ui.Th>[0]) => {
        let thRef: any = null;
        const [_, rest] = splitProps(props, [
          "header",
          "class",
          "classList",
          "children",
        ]);

        onMount(() => {
          // Note(sp): set the width of each <th> so that the width doesn't
          // change after first render until resized
          thRef.style.width =
            (config.columnWidths?.[props.header.id] || thRef.clientWidth) +
            "px";
        });
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
                ref={(node) => resizer(node, () => [props.header, thRef])}
              ></div>
            </div>
          </th>
        );
      },
    });
  };
};

export { withResizableColumns };
