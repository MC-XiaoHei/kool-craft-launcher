<template>
  <div class="absolute right-4 top-4 z-100 flex items-center opacity-80">
    <Button
      variant="ghost"
      class="size-9 no-drag hover:bg-transparent text-muted-foreground hover:text-foreground"
      @click="minimizeWindow"
    >
      <PhMinus class="h-5 w-5" weight="bold" />
    </Button>

    <Button
      variant="ghost"
      class="size-9 no-drag hover:bg-transparent text-muted-foreground hover:text-red-600 dark:hover:text-red-500"
      @click="closeWindow"
    >
      <PhX class="h-5 w-5" weight="bold" />
    </Button>
  </div>
</template>

<script setup lang="ts">
  import { Button } from "@/components/ui/button"
  import { PhMinus, PhX } from "@phosphor-icons/vue"
  import { getCurrentWindow } from "@tauri-apps/api/window"

  const appWindow = getCurrentWindow()

  async function minimizeWindow() {
    await appWindow.minimize()
  }

  async function closeWindow() {
    await appWindow.close()
  }
</script>
