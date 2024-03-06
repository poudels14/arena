import { HiSolidArrowLongDown } from "solid-icons/hi";
import Logo from "~/Logo";

export default function Home() {
  return (
    <main>
      <div class="h-full pt-20 pb-24 text-center dark:bg-gradient-to-b dark:from-slate-800 dark:from-20% dark:via-70% dark:to-gray-800">
        <div class="pb-4 justify-center items-baseline">
          <div class="pb-2 flex justify-center">
            <Logo size={150} />
          </div>
          <h1 class="pb-6 text-5xl font-medium text-gray-800 dark:text-gray-300">
            Portal
          </h1>
          <div class="pb-2 text-xl font-normal text-gray-800 dark:text-gray-400">
            All-in-one AI workspace
          </div>
        </div>
        <div class="flex justify-center">
          {/* <div class="px-6 py-3 text-base rounded-3xl text-white bg-indigo-600 cursor-pointer hover:underline">
              Get started
            </div> */}
          <a
            class="flex px-8 py-3 text-base justify-center items-center space-x-2 rounded-xl text-white bg-indigo-600"
            href="/download"
          >
            <div>Download now</div>
            <HiSolidArrowLongDown />
          </a>
        </div>
        <div class="flex py-3 justify-center text-xs text-gray-600">
          <div>Available for macOS and Linux</div>
        </div>
        <div class="text-xs font-normal text-gray-600 dark:text-gray-500">
          *offline support using Ollama
        </div>
      </div>

      {/* <div class="py-40 bg-slate-900/95">
          <div class="text-3xl font-medium text-gray-100">
            Go beyond just text
          </div>
          <div class="py-2 text-gray-500">
            Portal can generate dyanmic tables and charts
          </div>
        </div> */}

      {/* <div class="py-6 bg-gradient-to-r from-indigo-100 via-purple-100 to-pink-100">
          <div class="text-gray-700 text-3xl font-semibold">Use cases</div>
          <div class="py-2 text-gray-700">Your powerful AI assistant</div>
          <div class="py-40"></div>
        </div> */}

      {/* <ChatWithDocsSection /> */}
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
