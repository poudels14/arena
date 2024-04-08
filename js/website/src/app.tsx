// @refresh reload
import { Router } from "@solidjs/router";
import { FileRoutes } from "@solidjs/start";
import { Suspense } from "solid-js";
import "./app.css";
import NavigationBar from "./navigation";
import Footer from "./navigation/footer";

export default function App() {
  return (
    <div class="dark:bg-slate-900">
      <NavigationBar />
      <Router root={(props) => <Suspense>{props.children}</Suspense>}>
        <FileRoutes />
      </Router>
      <Footer />
    </div>
  );
}
