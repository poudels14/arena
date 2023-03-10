import { JSX, splitProps } from "solid-js";

type IconProps = {
  size: string;
  class?: string;
  classList?: Record<string, boolean>;
  ref?: any;
  /**
   * <path .../> should be passed as children
   */
  children: JSX.Element;
  onClick?: () => void;
};

/**
 * Inline icon
 *
 * Usage example:
 * import { AddIconPath } from "{icon-package}";
 * <InlineIcon size="20" name="add" children={AddIconPath} />
 */
const InlineIcon = (props: IconProps) => {
  const [local, restProps] = splitProps(props, ["size", "children"]);
  return (
    <svg
      width={local.size || "20"}
      height={local.size || "20"}
      viewBox={`0 0 20 20`}
      fill="currentColor"
      {...restProps}
    >
      {props.children}
    </svg>
  );
};

export { InlineIcon };
