import { createRouter, mergedRouter } from "@arena/runtime/server";
import * as chat from "./routes/chat";
import * as documents from "./routes/documents";
import * as plugins from "./routes/plugins";
import { p } from "./procedure";

const apiRouter = createRouter({
  prefix: "/api",
  routes: {
    "/chat/channels": chat.listChannels,
    "/chat/channels/:channelId": chat.getChannel,
    "/chat/:channelId/threads": chat.listThreads,
    "/chat/:channelId/threads/:threadId": chat.getThread,
    "/chat/:channelId/messages": chat.listMessages,
    "/chat/:channelId/messages/:id": chat.deleteMessage,
    "/chat/:channelId/send": chat.sendMessage,
    "/documents": documents.listDocuments,
    "/documents/:documentId": documents.getDocument,
    "/documents/:documentId/edit": documents.updateDocument,
    "/documents/:documentId/delete": documents.deleteDocument,
    // TODO(sagar): remove search path in production
    "/documents/search": documents.searchDocuments,
    "/documents/upload": documents.uploadDocuments,
    "/plugins": plugins.listPlugins,
    "/plugins/install": plugins.installPlugin,
    "/plugins/uninstall": plugins.uninstallPlugin,
    // TODO(sagar): remove search path in production
    "/plugins/search": plugins.searchPlugins,
  },
});

const workflowInterfaceRouter = createRouter({
  prefix: "/_system/interface/plugin/workflow",
  routes: {
    "/query": p.mutate(async ({ ctx, req }) => {
      // TODO
      return "";
    }),
  },
});

const router = mergedRouter({
  ignoreTrailingSlash: true,
  routers: [workflowInterfaceRouter, apiRouter],
  async middleware({ ctx, next }) {
    try {
      return await next({ ctx });
    } catch (e) {
      console.error(e);
      throw e;
    }
  },
  defaultHandler({ req }) {
    const url = new URL(req.url);
    if (url.pathname.startsWith("/api/")) {
      return new Response("404 Not found", {
        status: 404,
      });
    }
  },
});

export { router };
