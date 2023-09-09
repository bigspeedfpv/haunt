<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api";
import { useGameDataStore } from "@/lib/stores";
import { storeToRefs } from "pinia";

import PlayerCard from "@/components/PlayerCard.vue";
import { MatchData } from "@/lib/types";
import { useRouter } from "vue-router";

const store = useGameDataStore();

const { gameData } = storeToRefs(store);

const refresh = ref(30);
const router = useRouter();

function updateMatchStatus() {
  invoke<MatchData>("load_match")
    .then((res) => {
      if (res === gameData.value) return;

      console.log(`Match data updated: ${res}`);
      gameData.value = res;

      if (res.ingame) refresh.value = 30;
    })
    .catch((loggedIn: boolean) => {
      if (!loggedIn) {
        router.replace({ path: "/" });
      } else {
        router.replace({ path: "/pregame" });
      }

      return;
    });

  setTimeout(updateMatchStatus, refresh.value * 1000);
}

setTimeout(updateMatchStatus, refresh.value * 1000);
</script>

<template>
  <div
    class="grid w-full h-full gap-4 p-4"
    :style="{
      gridAutoFlow: 'column',
      gridTemplateColumns: `repeat(${Math.ceil(gameData!.players.length / 6)}, minmax(0, 1fr))`,
      gridTemplateRows: `repeat(${Math.ceil(gameData!.players.length / 2)}, minmax(0, 1fr))`
    }"
  >
    <PlayerCard
      v-for="player in gameData!.players"
      :key="player.name"
      :player="player"
    />
  </div>
</template>
