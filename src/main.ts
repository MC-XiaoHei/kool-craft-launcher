import { createApp } from "vue"
import { createPinia } from "pinia"
import App from "./App.vue"
import { attachConsole } from "@tauri-apps/plugin-log"
import "./style.css"
import router from "@/services/router"

await attachConsole()

const app = createApp(App)

app.use(router)
app.use(createPinia())

app.mount("#app")
