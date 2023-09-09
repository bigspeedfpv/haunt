<script setup lang="ts">
import { Player } from '@/lib/types';

const props = defineProps<{
  player: Player
}>()
</script>

<template>
  <div class="p-4 rounded-lg shadow-lg bg-black/5">
    <h1>{{ player.name }}</h1>
    <!-- if they don't have rank history they haven't played comp yet -->
    <span>{{ player.rankHistory[0]?.tierName ?? "UNRANKED" }}</span>
  </div>
</template>