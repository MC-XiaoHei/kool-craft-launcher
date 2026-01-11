import { createApp } from "vue"
import { createPinia } from "pinia"
import App from "./App.vue"
import { attachConsole } from "@tauri-apps/plugin-log"
import "./style.css"

await attachConsole()

const app = createApp(App)

app.use(createPinia())

app.mount("#app")
