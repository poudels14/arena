import { Body, Html } from "@arena/core/solid";

export default function Root() {
  return (
    <Html lang="en">
      <Body>
        <div class="h-screen bg-gray-100 flex items-center justify-around">
          <div class="font-bold text-4xl">Hello world!</div>
        </div>
      </Body>
    </Html>
  );
}
