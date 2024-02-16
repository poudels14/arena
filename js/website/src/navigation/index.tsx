export default function NavigationBar() {
  return (
    <nav class="sticky top-0 mx-auto text-center bg-slate-950/90 backdrop-blur-sm shadow-md shadow-slate-900">
      <div class="flex px-4 py-3 items-center space-x-16">
        <div class="px-4 text-xl font-bold text-gray-100">Portal</div>
        <div class="flex flex-1 text-gray-400 text-sm font-medium items-center space-x-5">
          <NavItem title="Features" />
          <NavItem title="Pricing" />
          <NavItem title="Contact" />
        </div>
        <div class="px-4 flex text-white text-sm items-center space-x-5">
          <a
            href="/login"
            class="px-5 py-1.5 rounded-2xl text-gray-400 cursor-pointer hover:underline"
          >
            Log in
          </a>
          <div class="px-5 py-1.5 rounded-2xl bg-indigo-600 cursor-pointer hover:bg-indigo-500">
            Sign up
          </div>
        </div>
      </div>
    </nav>
  );
}

const NavItem = (props: { title: string }) => {
  return <div class="cursor-pointer hover:underline">{props.title}</div>;
};
