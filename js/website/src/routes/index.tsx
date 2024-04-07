import { HiSolidArrowLongDown, HiOutlineArrowRight } from "solid-icons/hi";

export default function Home() {
  return (
    <main>
      <div class="h-full pt-20 pb-24 text-center dark:bg-gradient-to-b dark:from-slate-800 dark:from-20% dark:via-70% dark:to-gray-800">
        <div class="pt-36 pb-12 relative justify-center items-baseline">
          {/* <div class="pb-2 flex justify-center">
            <Logo size={150} />
          </div> */}
          {/* <h1 class="pb-6 text-5xl font-medium text-gray-800 dark:text-gray-300">
            Portal
          </h1> */}

          {/* <div class="pt-40"></div> */}
          <div class="absolute inset-0">
            <div class="pt-16 flex justify-center">
              <div class="flex py-2 px-5 text-sm rounded-full items-center space-x-3 cursor-pointer group border border-gray-50 hover:bg-gray-50">
                <div class="px-3 rounded-2xl bg-green-100 text-green-600">
                  New
                </div>
                <div class="">Introducing Portal Desktop</div>
                <HiOutlineArrowRight />
              </div>
            </div>
          </div>
          <div class="pb-2 text-5xl font-bold text-gray-800 dark:text-gray-400">
            A unified platform for AI applications
          </div>
        </div>
        <div class="flex justify-center">
          {/* <div class="px-6 py-3 text-base rounded-3xl text-white bg-indigo-600 cursor-pointer hover:underline">
              Get started
            </div> */}
          <a
            class="flex px-8 py-3 text-base justify-center items-center space-x-2 rounded-full text-white bg-indigo-600"
            href="/desktop"
          >
            <div>Download now</div>
            <HiSolidArrowLongDown />
          </a>
        </div>
        <div class="flex pt-2 pb-1 justify-center text-xs text-gray-600">
          <div>Available for macOS and Linux</div>
        </div>
        <div class="text-xs font-normal text-gray-600 dark:text-gray-500">
          *offline support using Ollama and LM Studio
        </div>
      </div>
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
