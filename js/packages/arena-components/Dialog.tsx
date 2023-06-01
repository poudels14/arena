import { Dialog as K } from "@kobalte/core";
import CrossIcon from "@blueprintjs/icons/lib/esm/generated-icons/20px/paths/cross";
import { InlineIcon } from "./InlineIcon";

type DialogProps = {
  title?: string;
  open: boolean;
  onOpenChange?: (isOpen: boolean) => void;
  children: any;
  /**
   * This can be used to control where to position the dialog window
   * by setting margin-top, etc
   */
  positionerClass?: string;
  contentClass?: string;
};

export default (props: DialogProps) => {
  return (
    <K.Root modal={true} open={props.open} onOpenChange={props.onOpenChange}>
      <K.Portal>
        <K.Overlay class="dialog-overlay fixed inset-0 bg-gray-200/80 backdrop-blur-sm" />
        <div
          class="dialog-positioner fixed inset-0 flex mt-28 justify-center"
          classList={{
            [props.positionerClass!]: Boolean(props.positionerClass),
          }}
        >
          <K.Content
            class="dialog-content max-w-[min(calc(100vw-16px),800px)] h-fit border border-gray-200 rounded-md p-4 pb-8 bg-white shadow-lg"
            classList={{
              [props.contentClass!]: Boolean(props.contentClass),
            }}
          >
            <div class="dialog-header flex align-baseline justify-between mb-3">
              <K.Title class="title text-xl font-medium text-accent-12">
                {props.title}
              </K.Title>
              <K.CloseButton class="close-button w-3 h-3 text-red-800 outline-none">
                <InlineIcon size="18px" class="cursor-pointer">
                  <path d={CrossIcon[0]} />
                </InlineIcon>
              </K.CloseButton>
            </div>
            <K.Description class="dialog-desciption">
              {props.children}
            </K.Description>
          </K.Content>
        </div>
      </K.Portal>
    </K.Root>
  );
};
