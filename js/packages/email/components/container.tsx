import { mergeProps, splitProps } from "solid-js";

const Container = (props: any) => {
  const [_, rest] = splitProps(props, ["style", "children"]);
  return (
    <table
      {...rest}
      // @ts-expect-error
      align="center"
      width="100%"
      data-id="__arena-email-container"
      role="presentation"
      cellSpacing="0"
      cellPadding="0"
      border={0}
      style={mergeProps({ "max-width": "37.5em" }, props.style)}
    >
      <tbody>
        <tr style={{ width: "100%" }}>
          <td>{props.children}</td>
        </tr>
      </tbody>
    </table>
  );
};

export { Container };
