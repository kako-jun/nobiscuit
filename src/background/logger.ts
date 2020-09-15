'use strict';

import _ from 'lodash';
import os from 'os';
import path from 'path';
import log4js from 'log4js';

class Logger {
  // class variables
  private static _logger: log4js.Logger;

  // instance variables

  constructor() {}

  public static initLogger(appName: string, { level = 'ALL', consoleEnabled = false }) {
    const appenders = ['fileLog'];
    if (consoleEnabled) {
      appenders.push('consoleLog');
    }

    const config = {
      appenders: {
        consoleLog: {
          type: 'console',
        },
        fileLog: {
          type: 'file',
          // {USER_HOME}/.{appName}/log/{appName}.log
          filename: path.join(os.homedir(), `.${appName}`, 'log', `${appName}.log`),
          maxLogSize: 5 * 1000 * 1000,
          backups: 5,
          keepFileExt: true,
        },
      },
      categories: {
        default: {
          level,
          appenders,
        },
      },
    };

    log4js.configure(config);
    this._logger = log4js.getLogger();
  }

  public static trace(...messages: any[]) {
    const line = this._genareteLine(messages);
    this._logger.trace(line);
  }

  public static debug(...messages: any[]) {
    const line = this._genareteLine(messages);
    this._logger.debug(line);
  }

  public static info(...messages: any[]) {
    const line = this._genareteLine(messages);
    this._logger.info(line);
  }

  public static warn(...messages: any[]) {
    const line = this._genareteLine(messages);
    this._logger.warn(line);
  }

  public static error(...messages: any[]) {
    const line = this._genareteLine(messages);
    this._logger.error(line);
  }

  public static fatal(...messages: any[]) {
    const line = this._genareteLine(messages);
    this._logger.fatal(line);
  }

  public static jsLog(arg: any) {
    switch (arg.level) {
      case 'trace':
        this._logger.trace(arg.messages);
        break;
      case 'debug':
        this._logger.debug(arg.messages);
        break;
      case 'info':
        this._logger.info(arg.messages);
        break;
      case 'warn':
        this._logger.warn(arg.messages);
        break;
      case 'error':
        this._logger.error(arg.messages);
        break;
      case 'fatal':
        this._logger.fatal(arg.messages);
        break;
    }
  }

  private static _genareteLine(messages: any[]) {
    // メッセージを結合する
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

    // ファイル名、メソッド名を追加する
    let fileName = '';
    let methodName = '';

    const stack = new Error().stack;
    if (stack) {
      // console.log(stack);
      const caller = stack.split('at ')[3].trim();
      // console.log(caller);
      const m = caller.match(/^(.+) \((.+?):.+:.+\)$/);
      if (m && m.length > 0) {
        fileName = path.basename(m[2]).split('?')[0];
        methodName = m[1];
      }
    }

    const line = `{file: "${fileName}",\tmethod: "${methodName}",\tmessages: [${margedMessage}]`;
    return line;
  }
}

export default Logger;
