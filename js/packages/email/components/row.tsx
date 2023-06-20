import { splitProps } from "solid-js";

const Row = (props: any) => {
  const [_, rest] = splitProps(props, ["children"]);
  return (
    <table
      {...rest}
      // @ts-expect-error
      align="center"
      width="100%"
      data-id="__arena-email-row"
      role="presentation"
      cellSpacing="0"
      cellPadding="0"
      border={0}
    >
      <tbody style={{ width: "100%" }}>
        <tr style={{ width: "100%" }}>{props.children}</tr>
      </tbody>
    </table>
  );
};

export { Row };
