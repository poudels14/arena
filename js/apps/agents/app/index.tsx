import { lazy } from "solid-js";

const Agents = lazy(() => import("./agents"));

const App = () => {
  return (
    <div>
      <div>
        <Agents />
      </div>
    </div>
  );
};

export default App;
