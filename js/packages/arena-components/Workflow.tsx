import {
  createSignal,
  Show,
  JSX,
  createContext,
  createSelector,
  Accessor,
  Setter,
  useContext,
} from "solid-js";

type CreateWorkflowProps = {
  /**
   * The first step to begin workflow from
   * Either an array of steps or a string of steps separated by "/"
   * should be passed
   */
  beginOn: string;

  /**
   * if `onNext` is passed, `next` callback should be called to actually go to
   * next step
   */
  onNext?: (
    /**
     * from: An id/slug of previous step
     *       `null` if it went to second step from the first step
     * to: An id/slug of next step
     * next: callback to actually go to next step
     */
    option: {
      from: string | null;
      to: string;
      data?: any;
      next: Workflow["next"];
    }
  ) => void;
  /**
   * If `onBack` is passed, `back` callback should be called to actully go back
   * to previous step
   */
  onBack?: (option: {
    from: string;
    to: string | null;
    back: Workflow["next"];
  }) => void;
  onComplete: (lastStep: string[]) => void;
  onCancel?: (lastStep: string[]) => void;
};

type Workflow = {
  config: {
    beginOn: string | string[];
  };
  getCurrentStep: Accessor<string>;
  // getPreviousStepData: Accessor<any>;
  /**
   * An array of data that was passed to next step in each steps
   * For example, if there are three steps, then data from 1->2 is
   * stored in stack[0], data from 2->3 is stored in stack[1]
   */
  stack: any[];
  next: (data?: any) => void;
  back: () => void;
  cancel: (data?: any) => void;
};

function createWorkflow(options: CreateWorkflowProps) {
  const [getCurrentStep, setCurrentStep] = createSignal(options.beginOn);
  const [getStepsData, setStepsData] = createSignal<any[]>([]);
  const currentStepSelector = createSelector(
    getCurrentStep,
    ([stepPath, exact]: any, curentPath: any) => {
      return exact ? stepPath == curentPath : curentPath?.startsWith(stepPath);
    }
  );

  /**
   * A collection of [fromStep, toStep]
   */
  const steps: [string, string][] = [];
  const addStepEdge = (from: string, to: string) => {
    steps.push([from, to]);
  };

  const workflow = Object.defineProperties(
    {
      config: { beginOn: options.beginOn },
      getCurrentStep,
      next(data?: any) {
        const currentStep = getCurrentStep();
        const [_, nextStep] = steps.find((s) => s[0] == currentStep) || null!;

        // if nextStep is null, complete the workflow
        if (nextStep == null) {
          return options.onComplete(data);
        }

        setStepsData((prev: any) => {
          return [data, ...prev];
        });

        if (options.onNext) {
          options.onNext({
            from: currentStep,
            to: nextStep,
            data,
            next() {
              setCurrentStep(nextStep);
            },
          });
        } else {
          setCurrentStep(nextStep);
        }
      },
      back() {
        const currentStep = getCurrentStep();
        const [prevStep, _] = steps.find((s) => s[1] == currentStep)!;
        setStepsData((prev: any) => {
          const [__, ...newPrev] = prev;
          return newPrev;
        });

        if (options.onBack) {
          options.onBack({
            from: currentStep,
            to: prevStep,
            back() {
              setCurrentStep(prevStep);
            },
          });
        } else {
          setCurrentStep(prevStep);
        }
      },
      cancel(data: any) {
        // TODO(sagar)
      },
    },
    {
      stack: {
        get: getStepsData,
      },
      setCurrentStep: {
        value: setCurrentStep,
        enumerable: false,
      },
      currentStepSelector: {
        value: currentStepSelector,
        enumerable: false,
      },
      addStepEdge: {
        value: addStepEdge,
        enumerable: false,
      },
    }
  );

  return [workflow, WorkflowComponent.bind(workflow)] as [
    Workflow,
    typeof WorkflowComponent
  ];
}

type InternalWorkflowContext = {
  config: any;
  getCurrentStep: Accessor<string>;
  setCurrentStep: Setter<string>;
  currentStepSelector: (key: any) => boolean;
  addStepEdge: any;
};

const InternalWorkflowContext = createContext<InternalWorkflowContext>();

type WorkflowProps = {
  children: any;
};

function WorkflowComponent(props: WorkflowProps) {
  // @ts-expect-error
  const self = this as unknown as Workflow & InternalWorkflowContext;
  return (
    <InternalWorkflowContext.Provider
      value={{
        config: self.config,
        getCurrentStep: self.getCurrentStep,
        setCurrentStep: self.setCurrentStep,
        currentStepSelector: self.currentStepSelector,
        addStepEdge: self.addStepEdge,
      }}
    >
      <div>{props.children}</div>
    </InternalWorkflowContext.Provider>
  );
}

const context: any = {
  isCurrentStep: function (exact?: boolean): boolean {
    return this.workflow.currentStepSelector([this.path, exact]);
  },
  useStep: function () {
    return this;
  },
};

type StepContext = {
  /**
   * Whether the current step is active/current
   */
  isCurrentStep: (exact?: boolean) => boolean;
  useStep: () => { id: string; path: string };
};

const StepContext = createContext<StepContext>();

type StepProps = {
  id: string;
  next: string | null;
  children: JSX.Element;
};

const Step = (props: StepProps) => {
  const { useStep } = useContext(StepContext)! || {};
  const workflow = useContext(InternalWorkflowContext)!;

  const step = {
    id: props.id,
    path: getStepPath(useStep, props.id),
    workflow,
  };

  workflow.addStepEdge(step.path, getStepPath(useStep, props.next));
  const isCurrentStep = context.isCurrentStep.bind(step);
  return (
    <StepContext.Provider
      value={{
        useStep: () => step,
        isCurrentStep,
      }}
    >
      <Show when={isCurrentStep()}>{props.children}</Show>
    </StepContext.Provider>
  );
};

function getStepPath<Id>(useStep: StepContext["useStep"], id: Id) {
  return useStep?.()?.path != undefined ? useStep().path + "/" + id || "" : id;
}

export { createWorkflow, Step };
