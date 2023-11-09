// import { pick } from "lodash-es";
// import { p } from "./procedure";

// const listWorkflows = p.query(async ({ ctx }) => {
//   const { default: sql } = ctx.dbs;

//   const { rows: workflowRuns } = await sql.query<any>(
//     `SELECT * FROM workflow_runs`
//   );
//   return workflowRuns.map((workflowRun) => {
//     return {
//       // ...pick(plugin, "id", "name", "description", "version"),
//       ...workflowRun,
//       plugin: JSON.parse(workflowRun.plugin),
//       triggeredAt: new Date(workflowRun.triggeredAt).toISOString(),
//       completedAt: workflowRun.completedAt
//         ? new Date(workflowRun.completedAt).toISOString()
//         : null,
//     };
//   });
// });

// const listenToUpdates = p.query(async ({ ctx, params }) => {
//   const { default: sql } = ctx.dbs;
//   const { id: workflowRunId } = params;

//   const {
//     rows: [workflowRun],
//   } = await sql.query<any>(`SELECT * FROM workflow_runs WHERE id = ?`, [
//     workflowRunId,
//   ]);

//   console.log("workflowRun =", workflowRun);

//   const stream = new ReadableStream({
//     async start(controller) {
//       setInterval(() => {
//         try {
//           controller.enqueue(
//             JSON.stringify({
//               updates: {
//                 msg: "Hello",
//               },
//             })
//           );
//         } catch (e) {}
//       }, 1000);
//     },
//     cancel() {
//       console.log("stream cancelled");
//     },
//   });

//   return new Response(stream, {
//     status: 200,
//     headers: [["content-type", "text/event-stream"]],
//   });
// });

// export { listWorkflows, listenToUpdates };
