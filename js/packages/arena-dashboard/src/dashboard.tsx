import { ChatBox } from "./jarvis";
import { Routes } from "./routes";
import { Sidebar } from "./sidebar";

const Dashboard = () => {
  return (
    <div class="flex">
      <div class="fixed w-52 flex flex-col left-0 top-0 bottom-0 text-sm">
        <Sidebar />
      </div>
      <main class="flex-1 fixed left-48 top-0 bottom-0 right-0">
        <Routes />
      </main>
      <div class="fixed left-0 bottom-0 right-0 py-2 flex justify-center pointer-events-none">
        <div class="w-[700px] pointer-events-auto">
          <ChatBox />
        </div>
      </div>
    </div>
  );
};

export { Dashboard };
