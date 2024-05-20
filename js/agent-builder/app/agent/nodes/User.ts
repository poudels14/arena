import { z, AgentNode, Context } from "@portal/cortex/agent";
import { AtLeastOne } from "@portal/cortex/agent/types";
import { Observable, ReplaySubject, Subject, concatMap } from "rxjs";

const config = z.object({});

const input = z.object({
  markdownStream: z.instanceof(Observable).label("Markdown stream"),
});

const output = z.object({
  markdownStream: z.instanceof(Observable).label("Markdown stream"),
});

export class User extends AgentNode<
  typeof config,
  typeof input,
  typeof output
> {
  #stream: Subject<any>;
  #subjectStreams: Observable<any>[];
  constructor() {
    super();
    this.#subjectStreams = [];
    this.#stream = new ReplaySubject();
  }

  get metadata() {
    return {
      id: "@core/user",
      version: "0.0.1",
      name: "User",
      config,
      input,
      output,
    };
  }

  init(context: Context<z.infer<typeof config>, z.infer<typeof output>>) {
    context.sendOutput({
      markdownStream: this.#stream.pipe(
        concatMap(() => this.#subjectStreams.pop()!)
      ),
    });
  }

  onInputEvent(
    context: Context<z.infer<typeof config>, z.infer<typeof output>>,
    data: AtLeastOne<z.infer<typeof input>>
  ) {
    if (data.markdownStream) {
      this.#subjectStreams.push(data.markdownStream);
      this.#stream.next(0);
    }
  }

  async *run(
    context: Context<
      z.infer<typeof this.metadata.config>,
      z.infer<typeof this.metadata.output>
    >,
    input: z.infer<typeof this.metadata.input>
  ) {
    if (input.markdownStream) {
      this.#subjectStreams.push(input.markdownStream);
      this.#stream.next(0);
    }
  }
}
