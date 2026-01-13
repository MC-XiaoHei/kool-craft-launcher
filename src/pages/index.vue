<script setup lang="ts">
  import Spacer from "@/components/ui/spacer/Spacer.vue"
  import { Button } from "@/components/ui/button"
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
        <div class="text-lg text-foreground">下午好，MC_XiaoHei。</div>
        <div class="text-sm text-muted-foreground">正在使用微软账号。</div>
      </div>
    </div>

    <Spacer />

    <div class="pl-10 flex flex-col justify-center items-start">
      <div class="text-6xl text-foreground font-extrabold">Survival Session 4</div>
    </div>

    <div class="pb-10 px-10 flex items-end h-28 w-full gap-4">
      <div class="text-sm text-muted-foreground">Fabric | 1.20.4</div>
      <Spacer />
      <Button class="size-16 rounded-xl shadow-md" variant="outline" />
      <Button class="h-16 w-48 rounded-xl shadow-md" />
    </div>
  </div>
</template>

<style scoped></style>
