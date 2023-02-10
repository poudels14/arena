import { ServerRoot, createHandler, renderAsync } from "@arena/core";

export default createHandler(
  renderAsync((event) => <ServerRoot event={event} />)
);