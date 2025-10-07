<template>
  <div class="flex items-center justify-center h-64">
    <div :class="['loader', variantClass]"></div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'


const variant = Math.floor(Math.random() * 5) + 1
const variantClass = computed(() => `loader-v${variant}`)
</script>

<style scoped>
/* Variant 1 */
.loader.loader-v1 {
  height: 60px;
  aspect-ratio: 1;
  display: grid;
  background:
    radial-gradient(farthest-side,#000 15%,#0000 18%),
    radial-gradient(50% 100% at 50% 160%,#fff 95%,#0000) top/100% 50%,
    radial-gradient(50% 100% at 50% -60%,#fff 95%,#0000) bottom/100% 50%;
  background-repeat: no-repeat;
}
.loader.loader-v1:before {
  content: "";
  background: inherit;
  opacity: 0.6;
  animation: l1 1s infinite;
}
@keyframes l1 {
  to {transform:scale(3);opacity:0}
}

/* Variant 2 */
.loader.loader-v2 {
  display: inline-flex;
  gap: 20px;
}
.loader.loader-v2:before,
.loader.loader-v2:after {
  content: "";
  height: 40px;
  aspect-ratio: 1;
  border-radius: 50%;
  background:
    radial-gradient(farthest-side,#000 95%,#0000) 35% 35%/12px 12px no-repeat
    #fff;
  animation: l5 3s infinite;
}
@keyframes l5 {
  0%,11%   {background-position:35% 35%}
  14%,36%  {background-position:65% 35%}
  38%,61%  {background-position:65% 65%}
  64%,86%  {background-position:35% 65%}
  88%,100% {background-position:35% 35%}
}

/* Variant 3 */
.loader.loader-v3 {
  display: inline-flex;
  gap: 20px;
}
.loader.loader-v3:before,
.loader.loader-v3:after {
  content: "";
  height: 40px;
  aspect-ratio: 1;
  border-radius: 50%;
  background:
    radial-gradient(farthest-side,#000 95%,#0000) 35% 35%/12px 12px no-repeat
    #fff;
  transform: scaleX(var(--s,1)) rotate(0deg);
  animation: l6 1s infinite linear;
}
.loader.loader-v3:after {
  --s: -1;
  animation-delay:-0.1s;
}
@keyframes l6 {
  100% {transform:scaleX(var(--s,1)) rotate(360deg);}
}

/* Variant 4 */
.loader.loader-v4 {
  display: inline-flex;
  gap: 20px;
}
.loader.loader-v4:before,
.loader.loader-v4:after {
  content: "";
  height: 40px;
  aspect-ratio: 1;
  border-radius: 50%;
  background:
    linear-gradient(#222 0 0) top/100% 40% no-repeat,
    radial-gradient(farthest-side,#000 95%,#0000) 50%/16px 16px no-repeat
    #fff;
  animation: 2.5s infinite;
  animation-name: l8-1, l8-2;
}
@keyframes l8-1 {
  0%,
  40%,
  100%{background-size:100% 40%,16px 16px}
  50%,
  80% {background-size:100% 0% ,16px 16px}
}
@keyframes l8-2 {
  0%,
  50% {background-position:top,50% 50%}
  60%,
  65%{background-position:top,70% 50%}
  70%,
  75%{background-position:top,30% 50%}
  90%,
  100%{background-position:top,50% 50%}
}

/* Variant 5 */
.loader.loader-v5 {
  display: inline-flex;
  gap: 20px;
}
.loader.loader-v5:before,
.loader.loader-v5:after {
  content: "";
  height: 40px;
  aspect-ratio: 1;
  border-radius: 50%;
  background:
    radial-gradient(farthest-side,#000 95%,#0000) 50%/16px 16px no-repeat
    #fff;
  animation: l10 1.5s infinite alternate;
}
.loader.loader-v5:after {
  --s:-1;
}
@keyframes l10 {
  0% ,20% {transform:scaleX(var(--s,1)) rotate(0deg) ;clip-path:inset(0)}
  60%,100%{transform:scaleX(var(--s,1)) rotate(30deg);clip-path:inset(40% 0 0)}
}
</style>


