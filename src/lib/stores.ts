import { defineStore } from "pinia";
import { ref } from "vue";
import { MatchData } from "./types";

export const useUserProfileStore = defineStore("userProfile", () => {
  const username = ref("pluh playa");
  const tagline = ref("boulets");

  return { username, tagline };
});

export const useGameDataStore = defineStore("gameData", () => {
  const gameData = ref<MatchData>();

  return { gameData };
});
