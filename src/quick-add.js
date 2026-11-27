import { mount } from "svelte";
import QuickAdd from "./pages/QuickAdd.svelte";
import ErrorBoundary from "./lib/components/ErrorBoundary.svelte";
import "./app.css";

mount(ErrorBoundary, {
  target: document.getElementById("app"),
  props: { component: QuickAdd },
});
