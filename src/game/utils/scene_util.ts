'use strict';

import _ from 'lodash';
import fs from 'fs';
import path from 'path';
import JsLogger from '../../utils/js_logger';

class SceneUtil {
  // class variables
  // instance variables

  // constructor() {}

  public static setBackgroundColor<T extends Phaser.Scene>(caller: T, color: string) {
    caller.cameras.main.setBackgroundColor(color);
  }
}

export default SceneUtil;
