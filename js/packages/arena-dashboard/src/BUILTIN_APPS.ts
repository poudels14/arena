// TODO(sagar): remove this once the app template manifests are
// uploaded to db after compilation
import type { TemplateManifest } from "@arena/sdk/app";
import AI_CHAT from "./@arena/ai-chat/MANIFEST";

const BUILTIN_APPS: TemplateManifest[] = [AI_CHAT];

export { BUILTIN_APPS };
