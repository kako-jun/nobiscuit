'use strict';
import electron from 'electron';
import { BrowserWindow } from 'electron';
const ipcMain = electron.ipcMain;

import _ from 'lodash';
import fs from 'fs';
import path from 'path';
import Logger from './logger';

class IPCListener {
  // instance variables
  private _win: BrowserWindow;
  private _isDevelopment: boolean;
  private _homeDirPath: string;

  constructor(win: BrowserWindow, isDevelopment: boolean) {
    this._win = win;
    this._isDevelopment = isDevelopment;

    this._homeDirPath = '';
    const homeDirPath = process.env[process.platform == 'win32' ? 'USERPROFILE' : 'HOME'];
    if (homeDirPath) {
      this._homeDirPath = homeDirPath;
    }
  }

  public listen() {
    Logger.info('listen');

    ipcMain.handle('jsLog', (event, arg: any) => {
      // Logger.info('jsLog called.');

      Logger.jsLog(arg);
    });

    ipcMain.handle('quit', () => {
      Logger.info('quit called.');

      electron.app.quit();
    });

    ipcMain.handle('getEnv', event => {
      Logger.info('getEnv called.');

      const env = {
        isDevelopment: this._isDevelopment,
        homeDirPath: this._homeDirPath,
      };

      Logger.info('env', env);
      return env;
    });

    ipcMain.handle('loadAppSetting', event => {
      Logger.info('loadAppSetting called.');

      let appSetting = null;
      const appSettingPath = path.resolve(this._homeDirPath, '.exiftence', 'app_setting.json');
      try {
        // fs.statSync(appSettingPath);
        appSetting = JSON.parse(fs.readFileSync(appSettingPath, { encoding: 'utf-8' }));
      } catch (e) {
        // do nothing.
      }

      Logger.info('appSetting', appSetting);
      return appSetting;
    });

    ipcMain.handle('saveAppSetting', (event, arg) => {
      Logger.info('saveAppSetting called.');

      const jsonStr = JSON.stringify(arg, null, 2);
      const appSettingPath = path.resolve(this._homeDirPath, '.exiftence', 'app_setting.json');
      try {
        fs.writeFileSync(appSettingPath, jsonStr, { encoding: 'utf-8' });
      } catch (e) {
        // do nothing.
      }
    });
  }
}

export default IPCListener;
