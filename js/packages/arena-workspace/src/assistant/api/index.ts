import { createRouter } from "@arena/core/router";
import * as chat from "./chat";
import * as documents from "./documents";
import * as plugins from "./plugins";
// import * as workflow from "./workflow";
// import * as workflowRuns from "./workflows";

const assistantRouter = createRouter({
  prefix: "/api/assistant",
  routes: {
    "/chat/channels": chat.listChannels,
    "/chat/channels/:channelId": chat.getChannel,
    "/chat/:channelId/threads": chat.listThreads,
    "/chat/:channelId/threads/:threadId": chat.getThread,
    // "/chat/:channelId/threads/:threadId/workflow/updates":
    //   chat.getWorkflowUpdates,
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
    // "/workflows": workflowRuns.listWorkflows,
    // "/workflows/:id/listen": workflowRuns.listenToUpdates,
    // "/workflow/:id": workflow.getWorkflow,
  },
});

// const workflowInterfaceRouter = createRouter({
//   prefix: "/_system/workflow/:workflowId",
//   routes: {
//     "/status": p.mutate(async ({ ctx, params, req }) => {
//       const body = (await req.json()) as {
//         messages: { role: string; content: string }[];
//       };

//       // TODO: Update workflow status
//       return [];
//     }),
//     "/chat/completion": p.mutate(async ({ ctx, params, req }) => {
//       const body = (await req.json()) as {
//         messages: { role: string; content: string }[];
//       };

//       const { response } = await chatCompletion({
//         userId: encodeToBase64(
//           Buffer.from(JSON.stringify({ workflowId: params.workflowId }))
//         ),
//         stream: false,
//         messages: body.messages,
//       }).catch((e) => {
//         return { response: e };
//       });

//       return new Response(JSON.stringify(response.data), {
//         status: response.status,
//       });
//     }),
//     "/chat/send": p.mutate(async ({ ctx, params, req, errors }) => {
//       const now = new Date();
//       const { default: sqlite } = ctx.dbs;
//       // TODO(sagar): verify that the workflow can send
//       // message to the given channel and thread
//       const body = (await req.json()) as {
//         channelId: string;
//         threadId: string;
//         message: { content: string };
//       };
//       console.log("body =", body);

//       console.log([params.workflowId, body.channelId, body.threadId]);
//       const {
//         rows: [workflowRun],
//       } = await sqlite.query<any>(
//         `SELECT * FROM workflow_runs WHERE id = ? AND channel_id = ? AND thread_id = ?`,
//         [params.workflowId, body.channelId, body.threadId]
//       );

//       if (!workflowRun) {
//         return errors.badRequest({ error: "Invalid workflow id" });
//       }

//       console.log("workflowRun =", workflowRun);

//       const messageId = uniqueId();

//       await sqlite.query(
//         `INSERT INTO chat_messages(id, channel_id, thread_id, role, user_id, message, timestamp)
//         VALUES (?,?,?,?,?,?,?)`,
//         [
//           messageId,
//           workflowRun.channelId,
//           workflowRun.threadId,
//           "workflow",
//           null,
//           JSON.stringify({
//             content: body.message.content,
//           }),
//           now.getTime(),
//         ]
//       );
//       return "Hello";
//     }),
//     "/llm/query": p.mutate(async ({ ctx, req }) => {
//       const body = await req.json();
//       console.log("body =", body);
//       return "Hello";
//     }),
//     "/query": p.mutate(async ({ ctx, req }) => {
//       const body = await req.json();
//       console.log("body =", body);
//       return "Hello";
//     }),
//   },
// });

export { assistantRouter };
