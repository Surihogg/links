import "./app.css";
import App from "./App.svelte";
import ErrorBoundary from "./lib/components/ErrorBoundary.svelte";
import { mount } from "svelte";

const app = mount(ErrorBoundary, {
  target: document.getElementById("app"),
  props: { component: App },
});

export default app;
