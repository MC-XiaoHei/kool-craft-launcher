import { createRouter, createWebHashHistory, type RouteRecordRaw } from "vue-router"
import { routes as autoRoutes } from "vue-router/auto-routes"

const customRoutes: RouteRecordRaw[] = []

const router = createRouter({
  history: createWebHashHistory(),
  routes: [...autoRoutes, ...customRoutes],
})

export default router
