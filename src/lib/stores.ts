import { defineStore } from "pinia";
import { ref } from "vue";
import { MatchData } from "./types";

export const useUserProfileStore = defineStore("userProfile", () => {
  const username = ref("pluh playa");
  const tagline = ref("boulets");
  const uuid = ref("1234");

  return { username, tagline, uuid };
});

export const useGameDataStore = defineStore("gameData", () => {
  const gameData = ref<MatchData>();

  return { gameData };
});
