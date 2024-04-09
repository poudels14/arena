const Footer = () => {
  return (
    <div class="px-12 md:px-56 pt-14 pb-24 flex text-xs space-x-10 text-gray-600 bg-slate-50">
      <div class="space-y-2">
        <div class="font-semibold text-gray-800">Product</div>
        <div>
          <a class="cursor-pointer hover:underline" href="/desktop">
            Portal Desktop
          </a>
        </div>
      </div>
      <div class="space-y-2">
        <div class="font-semibold text-gray-800">Community</div>
        <div>
          <a
            class="cursor-pointer hover:underline"
            href="https://discord.gg/kX4fYm7c"
          >
            Discord
          </a>
        </div>
      </div>
    </div>
  );
};

export default Footer;
