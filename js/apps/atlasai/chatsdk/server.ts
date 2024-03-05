import { ReplaySubject } from "rxjs";
import { ChatThread, Message } from "~/api/chat/types";

class ThreadOperationsStream {
  threadId: string;
  stream: ReplaySubject<any>;
  constructor(threadId: string, stream: ReplaySubject<any>) {
    this.threadId = threadId;
    this.stream = stream;
  }

  sendNewMessage(message: Message) {
    this.stream.next({
      ops: [
        {
          op: "replace",
          path: ["messages", message.id],
          value: message,
        },
      ],
    });
  }

  replaceMessageContent(messageId: string, value: string) {
    this.stream.next({
      ops: [
        {
          op: "replace",
          path: ["messages", messageId, "message", "content"],
          value: value,
        },
      ],
    });
  }

  // send the message chunk of the given messageId
  // this chunk is concatenated with existing chunks
  sendMessageChunk(messageId: string, value: string) {
    this.stream.next({
      ops: [
        {
          op: "add",
          path: ["messages", messageId, "message", "content"],
          value,
        },
      ],
    });
  }

  addNewThread(thread: ChatThread) {
    this.stream.next({
      ops: [
        {
          op: "replace",
          path: ["threads", thread.id],
          value: thread,
        },
      ],
    });
  }

  setThreadTitle(title: string) {
    this.stream.next({
      ops: [
        {
          op: "replace",
          path: ["threads", this.threadId, "title"],
          value: title,
        },
      ],
    });
  }

  close() {
    this.stream.complete();
  }
}

export { ThreadOperationsStream };
