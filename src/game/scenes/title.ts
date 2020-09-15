'use strict';

import _ from 'lodash';
import fs from 'fs';
import path from 'path';
import Phaser from 'phaser';
import JsLogger from '../../utils/js_logger';
import SceneUtil from '../utils/scene_util';
import ImageUtil from '../utils/image_util';
import TextUtil from '../utils/text_util';

class Title extends Phaser.Scene {
  // instance variables
  private map?: Phaser.Tilemaps.Tilemap;
  private tiles?: Phaser.Tilemaps.Tileset;
  private mapGroundLayer?: Phaser.Tilemaps.StaticTilemapLayer;

  private mapGround: number[][] = [
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0],
  ];

  constructor() {
    super({});
  }

  init() {
    JsLogger.trace('init.');
  }

  preload() {
    JsLogger.trace('preload.');
    this.load.image('mapTiles', '/assets/images/map_tile.png');
  }

  create() {
    JsLogger.trace('create.');
    SceneUtil.setBackgroundColor(this, '0xFFFFFF');

    this.map = this.make.tilemap({ data: this.mapGround, tileWidth: 32, tileHeight: 32 });
    this.tiles = this.map.addTilesetImage(`mapTiles`);
    this.mapGroundLayer = this.map.createStaticLayer(0, this.tiles, 0, 0);
    // const player = this.physics.add.image(200, 100, 'mapTiles');

    const text = TextUtil.showNovelText(this, 'Score: 0');
  }

  update() {
    // JsLogger.trace('update.');
  }
}

export default Title;
