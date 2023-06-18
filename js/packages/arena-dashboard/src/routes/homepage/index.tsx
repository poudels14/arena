import {
  Routes as SolidRoutes,
  Route,
  useLocation,
  useNavigate,
} from "@solidjs/router";
import { lazy, createEffect } from "solid-js";

const Homepage = () => {
  return (
    <div class="flex">
      <main class="flex-1">
        <div class="py-24">
          <div class="text-4xl font-bold text-center text-accent-12/90">
            Arena is in private beta. Join waitlist
          </div>
        </div>
      </main>
    </div>
  );
};

export default Homepage;
