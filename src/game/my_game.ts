'use strict';

import _ from 'lodash';
import fs from 'fs';
import path from 'path';
import Phaser from 'phaser';
import JsLogger from '../utils/js_logger';
import Preload from './scenes/preload';
import Title from './scenes/title';

class MyGame extends Phaser.Game {
  // instance variables
  constructor() {
    // const fpsConfig: Phaser.Types.Core.FPSConfig = {};
    const config: Phaser.Types.Core.GameConfig = {
      type: Phaser.AUTO,
      width: 800,
      height: 600,
      parent: 'content',
      physics: {
        default: 'arcade',
        arcade: {
          // debug: true,
          gravity: { y: 0 },
        },
        matter: {
          // debug: true,
          gravity: { y: 0.5 },
        },
      },
      // title: 'MyGame',
      // url: 'http://url.to.my.game',
      // version: '0.0.1',
      // fps: ,
    };

    super(config);

    this.scene.add('preload', Preload, false);
    this.scene.add('title', Title, false);

    this.scene.start('preload');
  }
}

export default MyGame;
