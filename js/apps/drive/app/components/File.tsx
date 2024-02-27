import { RiDocumentFileTextLine } from "solid-icons/ri";

const Directory = (props: {
  id: string;
  name: string;
  appId?: string;
  selected: boolean;
  onClick?: () => void;
  onDblClick?: () => void;
}) => {
  return (
    <Wrapper
      name={props.name}
      selected={props.selected}
      onClick={props.onClick}
      onDblClick={props.onDblClick}
    >
      <div class="absolute left-0 bottom-0 w-[50%] h-14 rounded-lg rounded-b-3xl rounded-tr-lg bg-blue-200"></div>
      <div class="absolute w-full h-[3.25rem] bottom-0 rounded-md rounded-b-3xl rounded-t-md bg-gradient-to-t from-blue-100 to-blue-200"></div>
      <div class="absolute right-0 bottom-0 w-[50%] h-[2.75rem] rounded-lg bg-slate-400"></div>
      <div class="absolute w-full h-10 bottom-0 rounded-lg bg-gradient-to-bl from-slate-400 to-slate-400"></div>
    </Wrapper>
  );
};

const File = (props: {
  id: string;
  name: string;
  type: string;
  selected: boolean;
  onClick?: () => void;
  onDblClick?: () => void;
}) => {
  return (
    <Wrapper
      name={props.name}
      selected={props.selected}
      onClick={props.onClick}
    >
      <div class="flex justify-center text-gray-600">
        <RiDocumentFileTextLine size={52} />
      </div>
    </Wrapper>
  );
};

const Wrapper = (props: {
  name: string;
  selected: boolean;
  onClick?: () => void;
  onDblClick?: () => void;
  children: any;
}) => {
  return (
    <div
      class="p-2 rounded cursor-pointer"
      classList={{
        "bg-indigo-100": props.selected,
        "hover:bg-indigo-50": !props.selected,
      }}
      onClick={props.onClick}
      onDblClick={props.onDblClick}
    >
      <div class="w-[4.5rem]">
        <div class="relative w-full h-14">{props.children}</div>
        <div class="px-1 py-1 w-full font-semibold leading-5 text-center line-clamp-2 text-gray-600">
          <div class="overflow-hidden text-ellipsis select-none">
            {props.name}
          </div>
        </div>
      </div>
    </div>
  );
};

export { Directory, File };
