import { mount, ClientRoot } from "@arena/core/client";
import Root from "~/root";

mount(() => <ClientRoot Root={Root} />, document);
