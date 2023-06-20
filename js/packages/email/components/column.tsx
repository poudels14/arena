import { splitProps } from "solid-js";

const Column = (props: any) => {
  const [_, rest] = splitProps(props, ["children"]);
  return (
    <td {...rest} data-id="__arena-email-column">
      {props.children}
    </td>
  );
};

export { Column };
