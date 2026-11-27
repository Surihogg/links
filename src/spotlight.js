import { mount } from "svelte";
import Spotlight from "./pages/Spotlight.svelte";
import ErrorBoundary from "./lib/components/ErrorBoundary.svelte";
import "./app.css";

mount(ErrorBoundary, {
  target: document.getElementById("app"),
  props: { component: Spotlight },
});
