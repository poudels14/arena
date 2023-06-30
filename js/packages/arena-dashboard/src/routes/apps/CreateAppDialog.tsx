import Dialog from "@arena/components/Dialog";
import CreateApp from "./create/index";

const CreateAppDialog = (props: { closeDialog: () => void }) => {
  return (
    <Dialog
      title="Create a new app"
      open={true}
      onOpenChange={(open) => !open && props.closeDialog()}
      contentClass="pt-4 px-4 w-[800px] shadow-accent-11"
    >
      <CreateApp onCreate={props.closeDialog} />
    </Dialog>
  );
};

export default CreateAppDialog;
