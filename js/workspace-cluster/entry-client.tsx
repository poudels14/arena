import { mount, ClientRoot } from "@portal/solidjs/client";
import Root from "./app/root";

mount(() => <ClientRoot Root={Root} />, document);
