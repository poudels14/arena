import { createWorkflow, Step } from "@arena/components/Workflow";
import SelectTemplate from "./SelectTemplate";
import ConfigureApp from "./ConfigureApp";

const CreateNewApp = (props: { onCreate: () => void }) => {
  const [workflow, Workflow] = createWorkflow({
    beginOn: "template",
    onNext({ data, next }) {
      next(data);
    },
    onComplete() {
      props.onCreate();
    },
  });

  return (
    <div>
      <Workflow>
        <Step id="template" next="configure">
          <SelectTemplate
            back={workflow.back}
            next={workflow.next}
            cancel={workflow.cancel}
          />
        </Step>
        <Step id="configure" next={null}>
          <ConfigureApp onCreate={workflow.next} template={workflow.stack[0]} />
        </Step>
      </Workflow>
    </div>
  );
};

export default CreateNewApp;
