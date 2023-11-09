// import { pick } from "lodash-es";
// import { p } from "./procedure";

// const workflow = {
//   status: "in-progress",
//   steps: [
//     {
//       name: "Step one",
//       status: "in-progress",
//       content: "This is a step one content",
//     },
//     {
//       name: "Step two",
//     },
//     {
//       name: "Step three",
//     },
//   ],
// };

// let i = 0;
// const getWorkflow = p.query(async ({ ctx }) => {
//   const { default: sql } = ctx.dbs;
//   // const { rows: plugins } = await sql.query<any>(`SELECT * FROM plugins`);

//   await new Promise((r) => {
//     setTimeout(() => {
//       r(null);
//     }, 500);
//   });

//   setTimeout(() => {
//     if (i < workflow.steps.length - 1) {
//       workflow.steps[i].status = "completed";
//       workflow.steps[i + 1].status = "in-progress";
//     } else {
//       workflow.steps[2].status = "completed";
//       workflow.status = "completed";
//     }
//     i++;
//   }, 500);

//   return workflow;
// });

// export { getWorkflow };
