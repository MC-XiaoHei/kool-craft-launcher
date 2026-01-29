<script setup lang="ts">
  import { useRoute } from "vue-router"
  import { PhGearSix, PhHouse, PhShoppingBag, PhSquaresFour } from "@phosphor-icons/vue"
  import { Button } from "@/components/ui/button"
  import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip"
  import { cn } from "@/lib/utils"

  const route = useRoute()

  const navItems = [
    { icon: PhHouse, label: "主页", path: "/" },
    { icon: PhSquaresFour, label: "实例库", path: "/library" },
    { icon: PhShoppingBag, label: "资源", path: "/resources" },
  ]

  const isActive = (path: string) => route?.path === path

  const handleNavClick = (e: MouseEvent) => {
    const target = e.currentTarget as HTMLElement
    target?.blur()
  }
</script>

<template>
  <nav class="flex h-full w-16 flex-col items-center py-2 z-50 shrink-0 bg-transparent">
    <div class="mb-6 flex items-center justify-center">
      <img class="size-10" src="/app-icon.svg" alt="icon" />
    </div>

    <div class="flex flex-col gap-4 w-full px-2 items-center">
      <TooltipProvider :delay-duration="1000">
        <template v-for="item in navItems" :key="item.path">
          <Tooltip>
            <TooltipTrigger as-child>
              <Button
                as-child
                variant="ghost"
                class="size-10! p-0 rounded-xl transition-all duration-200"
              >
                <RouterLink
                  :to="item.path"
                  @click="handleNavClick"
                  :class="
                    cn(
                      'flex items-center justify-center hover:bg-primary/10 hover:text-primary',
                      isActive(item.path)
                        ? 'bg-primary/10 text-primary'
                        : 'text-muted-foreground/60',
                    )
                  "
                >
                  <component :is="item.icon" class="size-6" weight="fill" />
                  <span class="sr-only">{{ item.label }}</span>
                </RouterLink>
              </Button>
            </TooltipTrigger>

            <TooltipContent
              side="right"
              :side-offset="10"
              class="bg-foreground text-background font-medium"
            >
              <p>{{ item.label }}</p>
            </TooltipContent>
          </Tooltip>
        </template>
      </TooltipProvider>
    </div>

    <div class="mt-auto flex flex-col items-center gap-4 w-full px-2">
      <TooltipProvider :delay-duration="100">
        <Tooltip>
          <TooltipTrigger as-child>
            <Button
              as-child
              variant="ghost"
              class="size-10! p-0 rounded-xl transition-all duration-200"
            >
              <RouterLink
                to="/settings"
                @click="handleNavClick"
                :class="
                  cn(
                    'flex items-center justify-center size-full rounded-xl hover:bg-primary/10 hover:text-primary',
                    isActive('/settings')
                      ? 'bg-primary/10 text-primary'
                      : 'text-muted-foreground/60',
                  )
                "
              >
                <PhGearSix class="size-6" weight="fill" />
              </RouterLink>
            </Button>
          </TooltipTrigger>
          <TooltipContent side="right" :side-offset="10">Settings</TooltipContent>
        </Tooltip>
      </TooltipProvider>
    </div>
  </nav>
</template>
