import { createHandler, renderAsync } from "@arena/core/server";
import { ServerRoot } from "@arena/core/solid";

export default createHandler(
  renderAsync((event) => <ServerRoot event={event} />)
);
