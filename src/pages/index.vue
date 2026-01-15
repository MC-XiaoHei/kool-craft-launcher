<script setup lang="ts">
  import { Button } from "@/components/ui/button"
  import { PhCube, PhFaders, PhPuzzlePiece, PhScroll } from "@phosphor-icons/vue"
  import { Separator } from "@/components/ui/separator"
  import { Spacer } from "@/components/ui/spacer"

  const instanceInfos = [
    { icon: PhCube, label: "1.20.4", key: "version" },
    { icon: PhScroll, label: "Fabric", key: "loader" },
    { icon: PhPuzzlePiece, label: "42 Mods", key: "mods" },
  ]
</script>

<template>
  <div class="flex flex-col items-start size-full">
    <div class="pt-14 pl-10 flex justify-center items-center h-14">
      <img
        class="size-14 rounded-md"
        src="https://avatars.githubusercontent.com/u/100807593?v=4"
        alt="avatar"
      />
      <div class="flex flex-col items-start justify-center pl-3 pb-1">
        <div class="text-lg text-foreground">下午好，MC_XiaoHei</div>
        <div class="text-sm text-muted-foreground">正在使用微软账号。</div>
      </div>
    </div>

    <Spacer />

    <div class="pl-10 flex flex-col justify-center items-start">
      <div class="text-6xl text-foreground font-extrabold">Survival Session 4</div>
    </div>

    <div class="pb-10 px-10 flex items-end h-26 w-full gap-4">
      <template v-for="(info, index) in instanceInfos" :key="info.label">
        <div class="flex items-center justify-center gap-2 group">
          <component
            :is="info.icon"
            class="text-muted-foreground size-5 group-hover:text-primary transition-colors"
            weight="fill"
          />
          <div class="text-lg font-bold leading-none">{{ info.label }}</div>

          <Separator
            v-if="index < instanceInfos.length - 1"
            orientation="vertical"
            class="mx-4 h-6!"
          />
        </div>
      </template>

      <Spacer />
      <Button
        class="size-16 rounded-xl shadow-sm transition-shadow duration-300 hover:shadow-lg hover:bg-background! dark:dark:bg-input/30! group"
        variant="outline"
      >
        <PhFaders
          class="size-8 rotate-90 text-muted-foreground group-hover:rotate-0 transition-transform duration-300"
          weight="bold"
        />
      </Button>
      <Button class="h-16 w-48 rounded-xl shadow-md hover:shadow-lg duration-300 text-secondary">
        <div class="text-2xl">启动游戏</div>
      </Button>
    </div>
  </div>
</template>

<style scoped></style>
