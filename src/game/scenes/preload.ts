'use strict';

import _ from 'lodash';
import fs from 'fs';
import path from 'path';
import Phaser from 'phaser';
import JsLogger from '../../utils/js_logger';
import SceneUtil from '../utils/scene_util';
import ImageUtil from '../utils/image_util';
import TextUtil from '../utils/text_util';

class Preload extends Phaser.Scene {
  // instance variables
  private _startText?: Phaser.GameObjects.Text;
  private fontStyle: Phaser.Types.GameObjects.Text.TextStyle = { color: 'red', fontSize: '70px' };

  constructor() {
    super({});
  }

  init() {
    JsLogger.trace('init.');
  }

  preload() {
    JsLogger.trace('preload.');
  }

  create() {
    JsLogger.trace('create.');
    SceneUtil.setBackgroundColor(this, '0xE08734');

    // this.add.text(400, 300, message, fontStyle);
    const text = TextUtil.showText(this, 'START', 400, 300, {
      color: 'red',
      fontSize: '70px',
      centering: true,
    });
    if (text) {
      this._startText = text;
    }

    if (this._startText) {
      this._startText.setInteractive();
      this._startText.on('pointerdown', () => {
        this.scene.start('title');
      });
    }
  }

  update() {
    // JsLogger.trace('update.');
  }
}

export default Preload;
