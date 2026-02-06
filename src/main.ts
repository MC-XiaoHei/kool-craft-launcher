import { createApp } from "vue"
import { createPinia } from "pinia"
import App from "./App.vue"
import router from "@/services/router"
import { i18n } from "@/services/i18n"
import { initLogger } from "@/services/log"

import "./style.css"
import "vue-color/style.css"

await initLogger()

const app = createApp(App)

app.use(router)
app.use(i18n)
app.use(createPinia())

app.mount("#app")
