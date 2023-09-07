<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api";
import { useUserProfileStore } from "@/stores/userProfile";
import { storeToRefs } from "pinia";

const store = useUserProfileStore();

const { username, tagline } = storeToRefs(store);

const refresh = ref(15);

// updates every second, but only refreshes data every `refresh` seconds
function checkMatchStatus() {
  if (refresh.value > 0) {
    refresh.value--;
    setTimeout(checkMatchStatus, 1000);
    return;
  }

  invoke("load_match")
    .then((res) => {
      console.log(res);
    })
    .catch(() => {
      // if no match found we check again in 5 secs
      refresh.value = 15;
      setTimeout(checkMatchStatus, 1000);
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
