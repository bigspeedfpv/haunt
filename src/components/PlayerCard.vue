<script setup lang="ts">
import { Player } from "@/lib/types";
import { computed } from "vue";

const props = defineProps<{
  player: Player;
}>();

// split name into display name and tag
const name = computed(() => props.player.name.split("#")[0]);
const tag = computed(() => props.player.name.split(/(?=#)/g)[1] ?? undefined);

const nameColor = computed(() =>
  props.player.team === "blue" ? "text-blue-300" : "text-red-300"
);
</script>

<template>
  <div
    class="p-4 rounded-lg shadow-lg bg-black/5 flex flex-row items-center gap-4"
  >
    <img
      :src="player.rankHistory[0]?.largeIcon ?? '/images/unranked.png'"
      alt="rank"
      class="w-16 h-16"
    />

    <div class="flex flex-col">
      <h1 :class="nameColor">
        <span class="font-bold text-xl">{{ name }}</span>
        <span v-if="tag" class="font-light">{{ tag }}</span>
      </h1>
    </div>
  </div>
</template>
