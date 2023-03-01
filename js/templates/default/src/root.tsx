import { Body, Html } from "@arena/core/solid";

export default function Root() {
  return (
    <Html lang="en">
      <Body
        style={
          "margin: 0px; background: rgba(100, 100, 100, 0.9); color: white;"
        }
      >
        <div
          style={
            "height: 100vh; display: flex; align-items: center; justify-content: space-around;"
          }
        >
          <div style={"font-size: 32;"}>Hello world!</div>
        </div>
      </Body>
    </Html>
  );
}
