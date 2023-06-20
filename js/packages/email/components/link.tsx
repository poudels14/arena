import { mergeProps, splitProps } from "solid-js";

const Link = (
  props: { target: string; style?: Record<string, string> } & any
) => {
  const [_, rest] = splitProps(props, ["target", "style"]);
  return (
    <a
      {...rest}
      data-id="__arena-email-link"
      target={props.target}
      style={mergeProps(
        {
          color: "#067df7",
          "text-decoration": "none",
        },
        props.style
      )}
    />
  );
};

export { Link };
