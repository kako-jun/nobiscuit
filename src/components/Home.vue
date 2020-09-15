<template>
  <div class="home">
    <div id="content"></div>
    <!-- <img src="/assets/images/map_tile.png" /> -->
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from 'vue-property-decorator';
import MyGame from '@/game/my_game';

@Component
export default class Home extends Vue {
  @Prop() private msg!: string;

  mounted() {
    new MyGame();
  }
}
</script>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style lang="scss" scoped>
.home {
  min-height: 100vh;
  display: flex;
  justify-content: center;
  align-items: center;
}

// #contenst canvas {
//   margin: 0;
// }
</style>
