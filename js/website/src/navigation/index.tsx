import PortalLogo from "../icons/portal.png";

export default function NavigationBar() {
  return (
    <nav class="sticky top-0 mx-auto text-center dark:bg-slate-900/90 backdrop-blur-sm shadow-sm dark:shadow-slate-800">
      <div class="flex px-4 py-3 items-center space-x-16">
        <a
          class="flex px-4 text-xl font-bold dark:text-gray-100 items-center space-x-2"
          href="/"
        >
          <img src={PortalLogo} width="32px" />
          <div>Portal</div>
        </a>
        <div class="flex flex-1 dark:text-gray-400 text-sm font-medium items-center space-x-5">
          {/* <NavItem title="Blog" href="/blog" /> */}
          <NavItem title="Discord" href="https://discord.gg/kX4fYm7c" />
          <NavItem title="Download" href="/desktop" />
          {/* 
          <NavItem title="Features" />
          <NavItem title="Pricing" />
          <NavItem title="Contact" /> */}
        </div>
        <div class="px-4 flex dark:text-white text-sm items-center space-x-4">
          <a
            href="/waitlist"
            class="px-5 py-1.5 rounded-2xl dark:text-gray-400 cursor-pointer hover:underline"
          >
            Sign up
          </a>
          <a
            href="/login"
            class="px-5 py-1.5 rounded-2xl text-white bg-indigo-600 cursor-pointer hover:bg-indigo-500"
          >
            Log in
          </a>
        </div>
      </div>
    </nav>
  );
}

const NavItem = (props: { title: string; href: string }) => {
  return (
    <a class="cursor-pointer hover:underline" href={props.href}>
      {props.title}
    </a>
  );
};
