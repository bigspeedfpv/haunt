<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";

const greetMsg = ref(false);
const name = ref("");

async function greet() {
  greetMsg.value = await invoke("login");
  await invoke("load_match");
}
</script>

<template>
  <div class="card">
    <input id="greet-input" v-model="name" placeholder="Enter a name..." />
    <button type="button" @click="greet()">Greet</button>
  </div>

  <p>{{ greetMsg }}</p>
</template>
