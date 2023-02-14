import { createHandler, renderAsync , ServerRoot} from "@arena/core/server";

export default createHandler(
  renderAsync((event) => <ServerRoot event={event} />)
);