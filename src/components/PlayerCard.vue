<script setup lang="ts">
import { Player } from "@/lib/types";
import { computed } from "vue";

const props = defineProps<{
  player: Player;
  partyColor?: string;
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
    class="overflow-clip rounded-lg shadow-lg bg-black/20 flex flex-row items-center"
  >
    <!-- party bar!!!! -->
    <div
      class="w-6 h-full flex justify-center items-center"
      :class="partyColor ?? 'bg-black/60'"
    >
      <h2 class="vertical-text font-bold">
        {{ player.character?.displayName.toUpperCase() ?? "UNDECIDED" }}
      </h2>
    </div>

    <img
      :src="player.character?.displayIcon"
      :alt="player.character?.displayName ?? 'Undecided'"
      :title="player.character?.displayName ?? 'Undecided'"
      class="h-full"
    />

    <!-- content -->
    <div class="flex justify-center items-center gap-2 p-4">
      <img
        :src="player.rankHistory[0]?.icon ?? '/images/unranked.png'"
        :alt="player.rankHistory[0]?.tierName ?? 'unranked'"
        :title="player.rankHistory[0]?.tierName ?? 'unranked'"
        class="h-12 w-12"
      />

      <div class="flex flex-col">
        <h1 :class="nameColor">
          <span class="font-bold text-xl">{{ name }}</span>
          <span v-if="tag" class="font-light">{{ tag }}</span>
        </h1>
      </div>
    </div>
  </div>
</template>

<style scoped>
.vertical-text {
  writing-mode: vertical-lr;
  transform: rotate(-180deg);
}
</style>
