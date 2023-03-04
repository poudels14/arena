import { createHandler, renderAsync } from "@arena/core/server";
import { ServerRoot } from "@arena/core/solid/server";

export default createHandler(
  renderAsync((event) => <ServerRoot event={event} />)
);
