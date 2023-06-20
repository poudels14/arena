import { mergeProps, splitProps } from "solid-js";
import { style } from "solid-js/web";

const Img = (props: {
  alt?: string;
  src: string;
  width: number;
  height: number;
  style?: Record<string, string>;
}) => {
  const [_, rest] = splitProps(props, [
    "alt",
    "src",
    "width",
    "height",
    "style",
  ]);
  return (
    <img
      {...rest}
      data-id="__arena-email-img"
      alt={props.alt}
      src={props.src}
      width={props.width}
      height={props.height}
      style={mergeProps(
        {
          display: "block",
          outline: "none",
          border: "none",
          "text-decoration": "none",
        },
        props.style
      )}
    />
  );
};

export { Img };
