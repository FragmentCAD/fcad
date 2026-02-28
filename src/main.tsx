import { render } from "preact";
import "./App.css"; // ¡Añadido para Tailwind y variables CSS!
import "./modules/core/lib/i18n"; // i18n initialization
import App from "./App";

render(<App />, document.getElementById("root")!);
