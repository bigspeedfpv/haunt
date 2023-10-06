<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";

import { useUserProfileStore } from "@/lib/stores";
import { storeToRefs } from "pinia";
import { useRouter } from "vue-router";

const router = useRouter();
const store = useUserProfileStore();

const { username, tagline, uuid } = storeToRefs(store);
const loginFail = ref("");

enum LoginFailType {
  Lockfile = 0,
  Entitlements,
  Session,
}

type UserInfo = {
  username: string;
  tag: string;
  uuid: string;
};

function refreshLogin() {
  invoke<UserInfo>("login")
    .then((res) => {
      console.log(res);
      username.value = res.username;
      tagline.value = res.tag;
      uuid.value = res.uuid;

      router.replace({ path: "/pregame" });
    })
    .catch((e) => {
      console.error(`Got login failure ${LoginFailType[e]}`);
      loginFail.value = LoginFailType[e];
    });
}

refreshLogin();
</script>

<template>
  <div class="w-full h-full flex flex-col items-center justify-center">
    <h1 class="text-5xl">
      Welcome to
      <span class="text-purple-400 font-bold">Haunt.</span>
    </h1>
    <span class="mt-3 text-2xl font-semibold opacity-80"
      >Waiting for Valorant...</span
    >
    <span class="mt-1 font-light opacity-50"
      >(Login failed: {{ loginFail }} not found)</span
    >
    <button
      @click="refreshLogin"
      class="mt-2 py-2 px-4 rounded-md bg-black/20 backdrop-saturate-150 shadow-lg shadow-black/20"
    >
      Refresh
    </button>
  </div>
</template>
