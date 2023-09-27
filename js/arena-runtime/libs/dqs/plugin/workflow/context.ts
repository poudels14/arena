import { Workflow } from "@arena/sdk/plugins/workflow";

type ContextProps = {
  /**
   * Workflow creator
   */
  creator: {
    app: {
      id: string;
    };
  };
};

const createChatContext = (props: ContextProps): Workflow.Context => {
  return {
    async confirm(messageTemplate, templateContext, schema) {
      return { confirmed: true };
    },
    async query<T>(msg, schema) {
      return {} as T;
    },
    async form(html) {
      return {};
    },
    async select<T>(msg, options) {
      return options.options[0];
    },
    async log(msg) {},
    async notify(msg) {},
  };
};

export { createChatContext };
