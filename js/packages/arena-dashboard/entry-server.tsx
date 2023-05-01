import { createHandler, renderAsync } from "@arena/core/server";
import { ServerRoot } from "@arena/core/solid/server";
import { router } from "~/api";

export default createHandler(
  router({
    prefix: "/api",
  }),
  renderAsync((event) => <ServerRoot event={event} />)
);
