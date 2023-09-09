<script setup lang="ts">
import { computed, ref } from "vue";
import { invoke } from "@tauri-apps/api";
import { useGameDataStore } from "@/lib/stores";
import { storeToRefs } from "pinia";

import PlayerCard from "@/components/PlayerCard.vue";

const store = useGameDataStore();

const { gameData } = storeToRefs(store);
</script>

<template>
  <div class="grid w-full h-full grid-flow-row grid-cols-2 gap-4 p-4">
    <PlayerCard v-for="player in gameData?.players" :key="player.name" :player="player" />
  </div>
</template>
