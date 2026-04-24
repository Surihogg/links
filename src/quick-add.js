import { mount } from "svelte";
import QuickAdd from "./pages/QuickAdd.svelte";
import "./app.css";

mount(QuickAdd, { target: document.getElementById("app") });
