import { Accessor, Show } from "solid-js";
import { Motion, Presence } from "@motionone/solid";
import { Portal } from "solid-js/web";

const SlidingDrawer = (props: {
  children: any;
  open?: Accessor<boolean>;
  onClose?: () => void;
  overlayClass?: string;
  contentClass?: string;
}) => {
  return (
    <Show when={props.open ? props.open() : true}>
      <Portal>
        <div class="drawer-component fixed top-0 bottom-0 right-0 left-0 w-full h-full">
          <div
            class="drawer-overlay absolute w-full h-full bg-gray-300/30 backdrop-blur-[2px]"
            classList={{
              [props.overlayClass!]: Boolean(props.overlayClass),
            }}
            onClick={props.onClose}
          ></div>
          <Presence>
            <Motion.div
              initial={{ opacity: 0, width: 0 }}
              animate={{ opacity: 1, width: "680px" }}
              exit={{ opacity: 0, width: "200px" }}
              transition={{ duration: 0.1, easing: "ease-in-out" }}
              class="drawer-content absolute right-0 w-0 h-full will-change-auto bg-white"
              classList={{
                [props.contentClass!]: Boolean(props.contentClass),
              }}
            >
              {props.children}
            </Motion.div>
          </Presence>
        </div>
      </Portal>
    </Show>
  );
};

export { SlidingDrawer };
