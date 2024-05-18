import { createRouter } from "@portal/server-core/router";

import { sendMessage } from "./message";

const router = createRouter({
  prefix: "/api",
  routes: {
    "/chat/sendMessage": sendMessage,
  },
});

export { router };
