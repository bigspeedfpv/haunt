<script setup lang="ts">
import { computed, ref } from "vue";
import { invoke } from "@tauri-apps/api";
import { useGameDataStore, useUserProfileStore } from "@/lib/stores";
import { storeToRefs } from "pinia";

import PlayerCard from "@/components/PlayerCard.vue";
import { MatchData, Player } from "@/lib/types";
import { useRouter } from "vue-router";

const userProfileStore = useUserProfileStore();
const gameDataStore = useGameDataStore();

const { uuid } = storeToRefs(userProfileStore);
const { gameData } = storeToRefs(gameDataStore);

const refresh = ref(15);
const router = useRouter();

function updateMatchStatus() {
  invoke<MatchData>("quick_update_match")
    .then((res) => {
      if (res === gameData.value) return;

      console.log("Match data updated", res);
      gameData.value = res;

      if (res.ingame) refresh.value = 60;
    })
    .catch((loggedIn: boolean) => {
      if (!loggedIn) {
        router.replace({ path: "/" });
      } else {
        router.replace({ path: "/pregame" });
      }

      return;
    });

  clearTimeout(timeout);
  timeout = setTimeout(updateMatchStatus, refresh.value * 1000);
}

let timeout = setTimeout(updateMatchStatus, refresh.value * 1000);

// switch teams if i'm on red
const teamSanitizedPlayers = computed(() => {
  const me = gameData.value!.players.find((p) => p.uuid === uuid.value)!;
  if (!me || me.team === "blue") return gameData.value!.players;

  const players: Player[] = [];

  for (const player of gameData.value!.players) {
    if (player.team === "blue") players.push({ ...player, team: "red" });
    else if (player.team === "red") players.push({ ...player, team: "blue" });
    else players.push(player);
  }

  return players;
});

function onKeyRefresh() {
  clearTimeout(timeout);
  updateMatchStatus();
}

const possibleColors = [
  "bg-fuchsia-400/75",
  "bg-rose-400/75",
  "bg-indigo-400/75",
  "bg-emerald-400/75",
  "bg-lime-400/75",
  "bg-cyan-400/75",
];

const partyColors = computed(() => {
  const partyOccurrences = new Map<string, number>();

  for (const player of gameData.value!.players) {
    const party = player.partyId;
    const occurrences = partyOccurrences.get(party) ?? 0;
    partyOccurrences.set(party, occurrences + 1);
  }

  const colors = new Map<string, string>();
  let partyColorIndex = 0; // we goin old school w this one!
  partyOccurrences.forEach((occurrences, party) => {
    if (party !== "" && occurrences > 1)
      colors.set(party, possibleColors[partyColorIndex++]);
  });

  console.log(partyOccurrences, colors);

  return colors;
});
</script>

<template>
  <div
    @keypress.r="onKeyRefresh"
    class="grid w-full h-full gap-4 p-4"
    :style="{
      gridAutoFlow: 'column',
      gridTemplateColumns: `repeat(${Math.ceil(teamSanitizedPlayers.length / 6)}, minmax(0, 1fr))`,
      gridTemplateRows: `repeat(${Math.ceil(teamSanitizedPlayers.length / 2)}, minmax(0, 1fr))`
    }"
  >
    <PlayerCard
      v-for="player in teamSanitizedPlayers"
      :key="player.uuid"
      :player="player"
      :partyColor="partyColors.get(player.partyId)"
    />
  </div>
</template>
