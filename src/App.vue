<template>
  <div id="app">
    <Home />
  </div>
</template>

<script lang="ts">
import { Component, Vue } from 'vue-property-decorator';
import Home from '@/components/Home.vue';
import JsLogger from '@/utils/js_logger';

@Component({
  components: {
    Home,
  },
})
export default class App extends Vue {
  async created() {
    // Electronかブラウザかを取得する
    let isBrowser = false;
    try {
      window.ipcRenderer.invoke('getEnv');
      // alert('electron');
    } catch (e) {
      isBrowser = true;
      // alert(e);
    }

    JsLogger.isBrowser = isBrowser;

    JsLogger.info('created');
    JsLogger.info('isBrowser', isBrowser);

    if (!isBrowser) {
      // デバッグ実行かを取得する
      const env = await window.ipcRenderer.invoke('getEnv');
      if (env) {
        JsLogger.info(env);
      }

      // 永続化ファイルから設定をリストアする
      const appSetting = await window.ipcRenderer.invoke('loadAppSetting');
      if (appSetting) {
        JsLogger.info(appSetting);
      }
    }
  }
}
</script>

<style lang="scss">
body {
  margin: 0;
}

#app {
  font-family: Avenir, Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-align: center;
  color: #2c3e50;
  background: darkgreen;
  overflow-x: hidden;
  overflow-y: hidden;
  user-select: none;
  min-height: 100vh;
}
</style>
