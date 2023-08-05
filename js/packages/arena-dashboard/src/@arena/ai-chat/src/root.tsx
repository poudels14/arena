import App from "./app";
import { Body, Html } from "@arena/core/solid";

export default function Root() {
  return (
    <Html lang="en">
      <Body class="antialiased">
        <App />
      </Body>
    </Html>
  );
}
