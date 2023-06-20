import { mergeProps, splitProps } from "solid-js";

const Hr = (props: { style: Record<string, string> } & any) => {
  const [_, rest] = splitProps(props, ["style"]);
  return (
    <hr
      {...rest}
      data-id="__arena-email-hr"
      style={mergeProps(
        {
          width: "100%",
          border: "none",
          "border-top": "1px solid #eaeaea",
        },
        props.style
      )}
    />
  );
};

export { Hr };
