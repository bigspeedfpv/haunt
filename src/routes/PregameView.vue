<script setup lang="ts">
const REFRESH_INTERVAL = 15;

import { ref } from "vue";
import { invoke } from "@tauri-apps/api";
import { useGameDataStore, useUserProfileStore } from "@/lib/stores";
import { storeToRefs } from "pinia";
import { MatchData } from "@/lib/types";
import { useRouter } from "vue-router";

const store = useUserProfileStore();

const { username, tagline } = storeToRefs(store);

const refresh = ref(REFRESH_INTERVAL);

const gameStore = useGameDataStore();
const router = useRouter();

let timeout: NodeJS.Timeout;

// updates every second, but only refreshes data every `refresh` seconds
function checkMatchStatus() {
  if (refresh.value > 0) {
    refresh.value--;
    clearTimeout(timeout);
    timeout = setTimeout(checkMatchStatus, 1000);
    return;
  }

  invoke<MatchData>("load_match")
    .then((res) => {
      console.log(res);
      gameStore.gameData = res;
      router.replace({ path: "/ingame" });
    })
    .catch((loggedIn: boolean) => {
      if (!loggedIn) {
        router.replace({ path: "/" });
      } else {
        refresh.value = REFRESH_INTERVAL;
        clearTimeout(timeout);
        timeout = setTimeout(checkMatchStatus, 1000);
      }
    });
}

checkMatchStatus();
</script>

<template>
  <div class="w-full h-full flex flex-col items-center justify-center">
    <h1 class="text-5xl">
      Hello,
      <span class="text-red-400 font-bold">{{ username }}</span>
      <span class="text-red-400 font-light text-3xl ml-1">#{{ tagline }} </span>
    </h1>

    <span class="mt-3 text-2xl font-semibold opacity-80"
      >Waiting for match...</span
    >
    <span class="mt-1 font-light opacity-50"
      >(Refreshing in {{ refresh }} seconds)</span
    >
  </div>
</template>
