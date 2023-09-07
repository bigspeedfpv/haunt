import { defineStore } from "pinia";
import { ref } from "vue";

export const useUserProfileStore = defineStore("userProfile", () => {
  const username = ref("pluh playa");
  const tagline = ref("boulets");

  return { username, tagline };
});
