import { HiSolidArrowLongDown, HiOutlineArrowRight } from "solid-icons/hi";
import PortalLogo from "../icons/portal.png";
import OpenaiLogo from "../assets/openai-logo.svg";
import AnthropicLogo from "../assets/anthropic-logo.svg";
import MistralLogo from "../assets/mistral-logo.webp";
import AppHomeScreenshot from "../assets/portal-desktop-screenshot.png";
import ChatWithDocsScreenshot from "../assets/chat-with-docs-screenshot.png";
import ChatProfilesScreenshot from "../assets/profiles-screenshot.png";

export default function Home() {
  return (
    <main class="*:py-36">
      <div class="h-full pt-12 md:pt-20 pb-10 text-center dark:bg-gradient-to-b dark:from-slate-800 dark:from-20% dark:via-70% dark:to-gray-800">
        <div class="pt-36 pb-12 relative justify-center items-baseline">
          <div class="absolute inset-0">
            <div class="pt-16 flex justify-center">
              <a
                class="flex py-2 px-5 text-sm rounded-full items-center space-x-3 cursor-pointer group border border-gray-50 hover:bg-gray-50"
                href="/desktop"
              >
                <div class="px-3 rounded-2xl bg-green-100 text-green-600">
                  New
                </div>
                <div class="">Introducing Portal Desktop</div>
                <HiOutlineArrowRight />
              </a>
            </div>
          </div>
          <div class="pb-2 text-2xl md:text-5xl font-bold text-gray-800 dark:text-gray-400">
            All in one AI assistant
          </div>
        </div>
        <div class="flex justify-center">
          <div>
            <DownloadButton />
            <div class="flex pt-2 pb-1 justify-center text-xs text-gray-600">
              <div>Available for macOS and Linux</div>
            </div>
            <div class="text-xs font-normal text-gray-600 dark:text-gray-500">
              *offline support using Ollama and LM Studio
            </div>
          </div>
        </div>
      </div>
      <AppScreenshot />
      <ChatWithDocsSection />
      <ChatProfilesSection />
      <LLMModelsSection />
      <DownloadPortalDesktop />
    </main>
  );
}

const DownloadButton = () => {
  return (
    <a
      class="flex max-w-52 px-6 py-3 text-base justify-center items-center space-x-2 rounded-full text-white bg-indigo-600"
      href="/desktop"
    >
      <div>Download</div>
      <HiSolidArrowLongDown />
    </a>
  );
};

const AppScreenshot = () => {
  return (
    <div class="px-12 md:px-56 bg-gradient-radial from-10% from-sky-50 via-sky-50/50 to-white/30 backdrop:blur-3xl">
      <div class="flex justify-center overflow-hidden rounded-lg bg-slate-100 border border-gray-200">
        <img
          src={AppHomeScreenshot}
          alt="Portal desktop screenshot"
          class="w-[800px]"
        />
      </div>
    </div>
  );
};

const ChatWithDocsSection = () => {
  return (
    <div class="flex justify-between px-12 md:px-56 space-x-10 bg-gradient-to-b from-sky-50/20 to-indigo-50/20">
      <div class="pt-10 space-y-4">
        <div class="text-3xl font-bold text-gray-700">
          Chat with your documents
        </div>
        <ul class="px-8 text-sm text-gray-500 list-disc space-y-2">
          <li>Upload PDF, Markdown, Microsoft docs, etc</li>
          <li>Organize documents into folders</li>
          <li>Narrow chat to a specific folder for better results</li>
        </ul>
      </div>
      <div>
        <img
          src={ChatWithDocsScreenshot}
          alt="Chat with docs screenshot"
          class="w-[500px]"
        />
      </div>
    </div>
  );
};

const ChatProfilesSection = () => {
  return (
    <div class="flex justify-between px-12 md:px-56 space-x-10 bg-gradient-radial from-10% from-indigo-50 via-indigo-50/50 to-white/30 backdrop:blur-3xl">
      <div>
        <img
          src={ChatProfilesScreenshot}
          alt="Chat with docs screenshot"
          class="w-[500px]"
        />
      </div>
      <div class="pt-10 space-y-4">
        <div class="text-3xl font-bold text-gray-700">
          Customize chat profiles
        </div>
        <ul class="px-8 text-sm text-gray-500 list-disc space-y-2">
          <li>Create unlimited chat profiles</li>
          <li>Instantly switch between chat profiles for better results</li>
        </ul>
      </div>
    </div>
  );
};

const LLMModelsSection = () => {
  const ModelProviderIcon = (props: { logo: any }) => {
    return (
      <div>
        <img src={props.logo} class="h-8" />
      </div>
    );
  };

  return (
    <div class="flex justify-center px-12 md:px-56 space-x-10 bg-gradient-to-b from-indigo-50/20 to-purple-50/10">
      <div class="space-y-4">
        <div class="flex items-center text-4xl font-bold text-gray-700 space-x-3">
          <div>Access state-of-the-art AI models</div>
          <div class="inline px-2 py-0.5 text-xs text-green-800 bg-green-50 border border-green-200 rounded">
            Coming soon
          </div>
        </div>
        <div class="px-8 text-center text-sm text-gray-500 space-y-2">
          Get access to the best AI models from OpenAI, Anthropic, Mixtral, etc
        </div>
        <div class="py-5 flex justify-center">
          <div class="space-y-6">
            <div class="flex justify-center">
              <img src={OpenaiLogo} class="h-8" />
            </div>
            <div class="flex justify-center">
              <img src={AnthropicLogo} class="h-4" />
            </div>
            <div class="flex justify-center">
              <img src={MistralLogo} class="h-7" />
            </div>
          </div>
          <div></div>
        </div>
      </div>
    </div>
  );
};

const DownloadPortalDesktop = () => {
  return (
    <div class="flex justify-center px-12 md:px-56 space-x-10 bg-gradient-to-b from-white to-purple-50/30">
      <div class="space-y-8">
        <div class="flex justify-center">
          <img src={PortalLogo} width="100px" />
        </div>
        <div class="text-3xl font-bold text-gray-700 space-y-6">
          <div>Download Portal Desktop for free</div>
          <div class="flex justify-center">
            <DownloadButton />
          </div>
        </div>
      </div>
    </div>
  );
};
