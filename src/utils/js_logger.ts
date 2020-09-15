'use strict';

import _ from 'lodash';
import path from 'path';

class JsLogger {
  // instance variables
  private static _isBrowser = false;

  // constructor() {}

  public static set isBrowser(isBrowser: boolean) {
    JsLogger._isBrowser = isBrowser;
  }

  // eslint-disable-next-line
  public static trace(...messages: any[]) {
    const line = this._genareteLine(messages);
    console.trace(line);
    if (!JsLogger._isBrowser) {
      window.ipcRenderer.invoke('jsLog', { level: 'trace', messages: line });
    }
  }

  // eslint-disable-next-line
  public static debug(...messages: any[]) {
    const line = this._genareteLine(messages);
    console.debug(line);
    if (!JsLogger._isBrowser) {
      window.ipcRenderer.invoke('jsLog', { level: 'debug', messages: line });
    }
  }

  // eslint-disable-next-line
  public static info(...messages: any[]) {
    const line = this._genareteLine(messages);
    console.info(line);
    if (!JsLogger._isBrowser) {
      window.ipcRenderer.invoke('jsLog', { level: 'info', messages: line });
    }
  }

  // eslint-disable-next-line
  public static warn(...messages: any[]) {
    const line = this._genareteLine(messages);
    console.warn(line);
    if (!JsLogger._isBrowser) {
      window.ipcRenderer.invoke('jsLog', { level: 'warn', messages: line });
    }
  }

  // eslint-disable-next-line
  public static error(...messages: any[]) {
    const line = this._genareteLine(messages);
    console.error(line);
    if (!JsLogger._isBrowser) {
      window.ipcRenderer.invoke('jsLog', { level: 'error', messages: line });
    }
  }

  // eslint-disable-next-line
  public static fatal(...messages: any[]) {
    const line = this._genareteLine(messages);
    console.error(line);
    if (!JsLogger._isBrowser) {
      window.ipcRenderer.invoke('jsLog', { level: 'fatal', messages: line });
    }
  }

  // eslint-disable-next-line
  private static _genareteLine(messages: any[]) {
    // メッセージを結合する
    // eslint-disable-next-line
    const messageStrings = _.map(messages, (m: any) => {
      if (_.isString(m)) {
        return `"${m.trim()}"`;
      }

      if (_.isError(m)) {
        return `\n${m.stack}\n`;
      }

      if (_.isObject(m) || _.isArray(m)) {
        // return JSON.stringify(m, null, 2);
        return JSON.stringify(m);
      }

      return String(m);
    });

    const margedMessage = messageStrings.join(',\t');

    // メソッド名、行数を追加する
    let fileName = '';
    let methodName = '';

    const stack = new Error().stack;
    if (stack) {
      // console.log(stack);
      const caller = stack.split('at ')[3].trim();
      // console.log(caller);
      // VueComponent.created (webpack-internal:///./node_modules/cache-loader/dist/cjs.js?!./node_modules/babel-loader/lib/index.js!./node_modules/ts-loader/index.js?!./node_modules/cache-loader/dist/cjs.js?!./node_modules/vue-loader/lib/index.js?!./src/App.vue?vue&type=script&lang=ts&:57:67)
      const m = caller.match(/^(.+) \(.+\/(.+?):.+\)$/);
      if (m && m.length > 0) {
        fileName = path.basename(m[2]).split('?')[0];
        methodName = m[1];
      }
    }

    const line = `{file: "${fileName}",\tmethod: "${methodName}",\tmessages: [${margedMessage}]`;
    return line;
  }
}

// JsLogger.isBrowser = true;

export default JsLogger;
