import { SiApple, SiLinux } from "solid-icons/si";

export default function Home() {
  return (
    <main>
      <div class="h-full pt-20 md:pt-32 pb-24 text-center dark:bg-gradient-to-b dark:from-slate-800 dark:from-20% dark:via-70% dark:to-gray-800">
        <div class="relative justify-center items-baseline">
          <div class="text-3xl md:text-5xl font-bold text-gray-700 dark:text-gray-400">
            Portal Desktop
          </div>
        </div>
        <div class="py-16 md:py-24 px-12 md:px-56 flex flex-col-reverse md:flex-row justify-between items-center space-y-reverse space-y-14 md:space-y-0 md:space-x-10 bg-gradient-to-b from-white to-purple-50/30">
          <div class="text-left space-y-5 text-gray-700">
            <div class="space-y-1.5">
              <div class="flex font-medium space-x-1">
                <div>Feature parity with Portal cloud</div>
                <sup class="inline-block mt-4 text-gray-400">*</sup>
              </div>
              <div class="text-xs text-gray-500">
                Enjoy same set of features as Portal Cloud
              </div>
            </div>
            <div class="space-y-1.5">
              <div class="font-medium">Bring your own model</div>
              <div class="text-xs text-gray-500">
                Use API from any model provider
              </div>
            </div>
            <div class="space-y-1.5">
              <div class="font-medium">Supports local models</div>
              <div class="text-xs text-gray-500">
                Use LMStudio and Ollama for offline support
              </div>
            </div>
            <div class="space-y-1.5">
              <div class="font-medium">Secure and privacy focused</div>
              <div class="text-xs text-gray-500">
                All you data stays in your computer (local AI models required)
              </div>
            </div>
          </div>
          <div class="text-3xl font-bold text-gray-700 space-y-6">
            <div class="space-y-4">
              <a
                class="flex px-8 py-3 text-xs justify-center items-center space-x-2 rounded-full text-white bg-indigo-600"
                href="/downloads/Portal_0.1.2_aarch64.dmg"
                target="_blank"
              >
                <SiApple />
                <div>Download for Mac</div>
              </a>
              <a
                class="flex px-8 py-3 text-xs justify-center items-center space-x-2 rounded-full text-white bg-indigo-600"
                href="/downloads/Portal_0.1.2_amd64.AppImage"
                target="_blank"
              >
                <SiLinux />
                <div>Download for Linux (AppImage)</div>
              </a>
              <a
                class="flex px-8 py-3 text-xs justify-center items-center space-x-2 rounded-full text-white bg-indigo-600"
                href="/downloads/Portal_0.1.2_amd64.deb"
                target="_blank"
              >
                <SiLinux />
                <div>Download for Linux (.deb)</div>
              </a>
            </div>
          </div>
        </div>
      </div>
    </main>
  );
}
