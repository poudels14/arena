import { SiApple, SiLinux } from "solid-icons/si";

export default function Home() {
  return (
    <main>
      <div class="h-full pt-32 pb-24 text-center dark:bg-gradient-to-b dark:from-slate-800 dark:from-20% dark:via-70% dark:to-gray-800">
        <div class="pb-12 relative justify-center items-baseline">
          <div class="pb-2 text-4xl font-bold text-gray-700 dark:text-gray-400">
            Portal Desktop
          </div>
          <div class="text-xs font-normal text-gray-600 dark:text-gray-500">
            *offline support using Ollama and LM Studio
          </div>
        </div>
        <div class="flex justify-center">
          <div class="space-y-4">
            <a
              class="flex px-8 py-3 text-sm justify-center items-center space-x-2 rounded-full text-white bg-indigo-600"
              href="/downloads/mac"
              target="_blank"
            >
              <SiApple />
              <div>Download for Mac</div>
            </a>
            <a
              class="flex px-8 py-3 text-sm justify-center items-center space-x-2 rounded-full text-white bg-indigo-600"
              href="/downloads/linux-appimage"
              target="_blank"
            >
              <SiLinux />
              <div>Download for Linux (AppImage)</div>
            </a>
            <a
              class="flex px-8 py-3 text-sm justify-center items-center space-x-2 rounded-full text-white bg-indigo-600"
              href="/downloads/linux-deb"
              target="_blank"
            >
              <SiLinux />
              <div>Download for Linux (.deb)</div>
            </a>
          </div>
        </div>
      </div>
    </main>
  );
}

const ChatWithDocsSection = () => {
  return (
    <div class="pb-48 ">
      <div class="text-2xl font-medium text-gray-700">
        Chat with your documents
      </div>
    </div>
  );
};
