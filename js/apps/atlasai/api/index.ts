import { createRouter } from "@portal/server-core/router";
import * as chat from "./chat";
import * as artifacts from "./artifacts";

const router = createRouter({
  prefix: "/api/",
  ignoreTrailingSlash: true,
  async middleware({ ctx, next }) {
    try {
      return await next({ ctx });
    } catch (e) {
      console.error(e);
      throw e;
    }
  },
  routes: {
    "/chat/threads/": chat.listThreads,
    "/chat/threads/:threadId": chat.getThread,
    "/chat/threads/:threadId/delete": chat.deleteThread,
    "/chat/threads/:threadId/messages/:id": chat.deleteMessage,
    "/chat/threads/:threadId/send": chat.sendMessage,
    "/chat/threads/:threadId/tasks/": chat.listActiveTasks,
    "/chat/tasks/": chat.listActiveTasks,
    "/chat/artifacts/:id": artifacts.getArtifact,
    "/chat/artifacts/:id/content": artifacts.getArtifactContent,
    // "/documents": documents.listDocuments,
    // "/documents/:documentId": documents.getDocument,
    // "/documents/:documentId/edit": documents.updateDocument,
    // "/documents/:documentId/delete": documents.deleteDocument,
    // TODO(sagar): remove search path in production
    // "/documents/search": documents.searchDocuments,
    // "/documents/upload": documents.uploadDocuments,
    // "/plugins": plugins.listPlugins,
    // "/plugins/install": plugins.installPlugin,
    // "/plugins/uninstall": plugins.uninstallPlugin,
    // // TODO(sagar): remove search path in production
    // "/plugins/search": plugins.searchPlugins,
  },
});

export { router };
