import { mount, ClientRoot } from "@arena/core/solid/client";
import { setupTwind } from "@arena/uikit/twind";

if (process.env.MODE === "development") {
  setupTwind({});
}

mount(() => <ClientRoot />, document);
