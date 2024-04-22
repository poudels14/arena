import { render } from "solid-js/web";
import * as Sentry from "@sentry/browser";
import SplashScreen from "./app/SplashScreen";

Sentry.init({
  dsn: "https://b6d70976adbb0932725f7b4817e422cf@o4507128581914624.ingest.us.sentry.io/4507131496366080",
  integrations: [],
  tracesSampleRate: 1.0,
  profilesSampleRate: 0.0,
});

render(() => <SplashScreen />, document.body);
