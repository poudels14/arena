import { splitProps } from "solid-js";

const Section = (props: any) => {
  const [_, rest] = splitProps(props, ["children"]);
  return (
    <table
      {...rest}
      // @ts-expect-error
      align="center"
      width="100%"
      data-id="__arena-email-section"
      border={0}
      cellPadding="0"
      cellSpacing="0"
      role="presentation"
    >
      <tbody>
        <tr>
          <td>{props.children}</td>
        </tr>
      </tbody>
    </table>
  );
};

export { Section };
