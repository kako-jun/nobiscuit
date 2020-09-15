'use strict';

import _ from 'lodash';
import fs from 'fs';
import path from 'path';
import JsLogger from '../../utils/js_logger';

class TextUtil {
  // class variables
  // instance variables

  // constructor() {}

  public static showText<T extends Phaser.Scene>(
    caller: T,
    message: string,
    x: number,
    y: number,
    { color = '', fontSize = '', centering = false },
  ) {
    const fontStyle: Phaser.Types.GameObjects.Text.TextStyle = { color, fontSize };
    const text = caller.add.text(x, y, message, fontStyle);

    if (centering) {
      text.setOrigin(0.5, 0.5);
    }

    return text;
  }

  public static showNovelText<T extends Phaser.Scene>(caller: T, message: string) {
    const messages = _.map(message, (c, i) => {
      return message.slice(0, message.length - i);
    });

    _.forEach(messages, m => {
      const text = this.showText(caller, m, 12, 12, {
        color: 'black',
        fontSize: '24px',
      });

      return text;
    });
  }
}

export default TextUtil;
