import NavigationBar from "../navigation";

export default function Home() {
  return (
    <div class="h-screen font-sans bg-slate-900">
      <NavigationBar />
      <main class="h-[calc(100%-theme(spacing.14))] mx-auto text-center">
        <div class="h-full pt-32 bg-gradient-to-b from-slate-950 from-20% via-70% to-gray-900">
          <div class="pb-4  justify-center items-baseline">
            <h1 class="pb-2 text-5xl font-medium text-gray-300">Portal</h1>
            <div class="pb-16 text-xl font-normal text-gray-500">
              all in one AI workspace
            </div>
          </div>
          <div class="flex justify-center">
            <div class="px-6 py-3 text-base rounded-3xl text-white bg-indigo-600 cursor-pointer hover:underline">
              Get started
            </div>
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
      </main>
    </div>
  );
}
