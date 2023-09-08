import { A } from "@solidjs/router";
import { InlineIcon } from "@arena/components";
import { DropdownMenu } from "@kobalte/core";
import ThreeDotsIcon from "@blueprintjs/icons/lib/esm/generated-icons/20px/paths/more";

const AppThumbnail = (props: {
  id: string;
  name: string;
  description?: string;
  config: any;
  delete: (id: string) => void;
}) => {
  const thumbnailClass = () => props.config?.ui?.thumbnail?.class;
  return (
    <A
      href={"/apps/" + props.id}
      class="w-48 h-24 lg:w-64 lg:h-36 relative group bg-brand-2 rounded-lg bg-gradient-to-br cursor-pointer"
      classList={{
        [thumbnailClass()]: Boolean(thumbnailClass()),
      }}
    >
      <div class="absolute bottom-0 w-full flex pl-4 pr-2 py-2">
        <div class="flex-1 font-medium text-accent-12/80 group-hover:text-brand-12">
          {props.name}
        </div>
        <MenuDropdown delete={props.delete} />
      </div>
    </A>
  );
};

const MenuDropdown = (props: { delete: (id: string) => void }) => {
  return (
    <DropdownMenu.Root
      hideWhenDetached={true}
      gutter={2}
      shift={-25}
      modal={true}
    >
      <DropdownMenu.Trigger class="px-2 appearance-none inline-flex text-brand-12/80 items-center outline-none gap-8">
        <InlineIcon size="12px">
          <path d={ThreeDotsIcon[0]} />
        </InlineIcon>
      </DropdownMenu.Trigger>
      <DropdownMenu.Portal>
        <DropdownMenu.Content
          class="p-1 min-w-[160px] bg-white text-xs outline-none rounded-md border border-brand-12/30 shadow-lg"
          onChange={() => {
            // TODO(sagar): figure out how to handle item click
          }}
        >
          <DropdownItem label="Share" />
          <DropdownItem label="Delete" onClick={props.delete} />
        </DropdownMenu.Content>
      </DropdownMenu.Portal>
    </DropdownMenu.Root>
  );
};

const DropdownItem = (props: any) => {
  return (
    <DropdownMenu.Item
      class="p-2 hover:bg-brand-10/10 cursor-pointer rounded"
      onSelect={props.onClick}
    >
      <div>{props.label}</div>
    </DropdownMenu.Item>
  );
};

export default AppThumbnail;
