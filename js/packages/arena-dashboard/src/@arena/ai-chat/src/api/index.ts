import { createRouter } from "@arena/runtime/server";
import * as chat from "./routes/chat";
import * as documents from "./routes/documents";

const router = createRouter({
  prefix: "/api",
  async middleware({ ctx, next }) {
    try {
      return await next({ ctx });
    } catch (e) {
      console.error(e);
      throw e;
    }
  },
  routes: {
    "/chat/channels": chat.listChannels,
    "/chat/:channelId/messages": chat.listMessages,
    "/chat/:channelId/send": chat.sendMessage,
    "/chat/:channelId/messages/:id": chat.deleteMessage,
    "/documents": documents.listDocuments,
    "/documents/:documentId": documents.getDocument,
    "/documents/:documentId/edit": documents.updateDocument,
    "/documents/:documentId/delete": documents.deleteDocument,
    "/documents/search": documents.searchDocuments,
    "/documents/upload": documents.uploadDocuments,
  },
});

export { router };
