import { FileExplorer } from "./FileExplorer";
import { UploadTracker, UploadTrackerProvider } from "./UploadTracker";

const App = () => {
  return (
    <div>
      <UploadTrackerProvider>
        <FileExplorer />
        <UploadTracker />
      </UploadTrackerProvider>
    </div>
  );
};

export default App;
