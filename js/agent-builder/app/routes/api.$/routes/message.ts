import { ReplaySubject } from "rxjs";
import { uniqueId } from "@portal/cortex/utils/uniqueId";
import { z } from "@portal/server-core";

import { p } from "./procedure";
import { DefaultExecutorBuilder } from "../../../agent/executor";
import { AgentMetadata } from "../../../editor/schema";
import { loadAgentGraph } from "../trpc/agent";

const sendMessage = p
  .input(
    z.object({
      id: z.string(),
      agentId: z.string(),
      message: z.object({
        content: z.string(),
      }),
    })
  )
  .mutate(async ({ body }) => {
    const replayStream = new ReplaySubject<any>();

    let responseMessageId = uniqueId();
    replayStream.next({
      type: "message" as const,
      data: {
        id: responseMessageId,
        message: {
          content: "",
        },
        role: "ai",
        createdAt: new Date().toISOString(),
      },
    });

    const agent = await loadAgentGraph(body.agentId);
    await executeGraph(responseMessageId, replayStream, body.message, agent);
    const responseStream = new ReadableStream({
      async start(controller) {
        replayStream.subscribe({
          next(data) {
            try {
              controller.enqueue("data: " + JSON.stringify(data) + "\n\n");
            } catch (e) {
              console.error("Error sending message to stream:", e);
            }
          },
          complete() {
            try {
              controller.close();
            } catch (e) {
              console.error("Error closing stream:", e);
            }
          },
        });
      },
    });
    return new Response(responseStream, {
      status: 200,
      headers: [["Content-Type", "text/event-stream"]],
    });
  });

const executeGraph = async (
  messageId: string,
  subject: ReplaySubject<any>,
  message: any,
  agent: AgentMetadata
) => {
  const builder = new DefaultExecutorBuilder();
  const executor = builder.compileGraph(agent.graph);
  const stream = executor.newExecutionStream();

  stream.subscribe((event) => {
    console.log("EVENT =", event);
    if (event.type == "output" && event.node.id == "user") {
      if (event.output.stream) {
        event.output.stream.subscribe((data: any) => {
          if (data.type == "content/delta") {
            subject.next({
              type: "message/delta" as const,
              data: {
                id: messageId,
                message: {
                  content: {
                    delta: data.delta,
                  },
                },
              },
            });
          }
        });
      }
    }
  });

  stream.trigger(
    { id: "input" },
    {
      // query: "Who are you?",
      // query: "Who am I?",
      query: message.content,
      threadId: null,
    }
  );
};

export { sendMessage };
