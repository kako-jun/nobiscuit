import Phaser from 'phaser';
import { MazeGenerator } from '../utils/MazeGenerator';

export default class MazeScene extends Phaser.Scene {
  // 迷路データ (1=壁, 0=通路, 2=ゴール, 3=看板)
  private maze: number[][] = [];
  private signs: Map<string, string> = new Map();

  // プレイヤー位置と向き（グリッドベース）
  private playerX: number = 1; // グリッド位置（整数）
  private playerY: number = 1; // グリッド位置（整数）
  private playerDir: number = 0; // 方向 0=東, 1=南, 2=西, 3=北

  // レイキャスティング設定
  private readonly FOV = Math.PI / 3; // 視野角60度
  private readonly NUM_RAYS = 120; // レイ数（解像度）
  private readonly MAX_DEPTH = 20; // 最大描画距離

  // グラフィックス
  private graphics!: Phaser.GameObjects.Graphics;
  private minimapGraphics!: Phaser.GameObjects.Graphics;
  private signText!: Phaser.GameObjects.Text;

  // 入力
  private cursors!: Phaser.Types.Input.Keyboard.CursorKeys;
  private wasd: any = {};

  // キーリピート防止
  private lastKeyPressTime: number = 0;
  private keyRepeatDelay: number = 200; // ms

  // 看板表示状態
  private currentSignMessage: string = '';
  private signDisplayDistance: number = 1.5; // 看板が表示される距離

  constructor() {
    super({ key: 'MazeScene' });
  }

  create() {
    // 迷路を生成
    const generator = new MazeGenerator(16, 13);
    const result = generator.generate();
    this.maze = result.maze;
    this.signs = result.signs;

    this.graphics = this.add.graphics();
    this.minimapGraphics = this.add.graphics();

    // 看板メッセージ用テキスト
    this.signText = this.add.text(
      this.scale.width / 2,
      this.scale.height - 100,
      '',
      {
        fontSize: '32px',
        color: '#FF4444',
        backgroundColor: '#000000',
        padding: { x: 20, y: 10 },
        stroke: '#FFFFFF',
        strokeThickness: 2,
      }
    );
    this.signText.setOrigin(0.5);
    this.signText.setVisible(false);

    // キーボード入力
    this.cursors = this.input.keyboard!.createCursorKeys();
    this.wasd = {
      up: this.input.keyboard!.addKey(Phaser.Input.Keyboard.KeyCodes.W),
      down: this.input.keyboard!.addKey(Phaser.Input.Keyboard.KeyCodes.S),
      left: this.input.keyboard!.addKey(Phaser.Input.Keyboard.KeyCodes.A),
      right: this.input.keyboard!.addKey(Phaser.Input.Keyboard.KeyCodes.D),
    };

    // タッチ操作エリアを4分割
    this.input.on('pointerdown', (pointer: Phaser.Input.Pointer) => {
      const now = Date.now();
      if (now - this.lastKeyPressTime < this.keyRepeatDelay) return;
      this.lastKeyPressTime = now;

      const centerX = this.scale.width / 2;
      const centerY = this.scale.height / 2;

      if (pointer.y < centerY) {
        // 上半分：前進
        this.moveForward();
      } else {
        // 下半分：後退
        this.moveBackward();
      }

      if (pointer.x < centerX) {
        // 左半分：左回転
        this.turnLeft();
      } else {
        // 右半分：右回転
        this.turnRight();
      }
    });
  }

  update(time: number, delta: number) {
    this.handleInput();
    this.checkNearbySign();
    this.render3DView();
    this.renderMinimap();
  }

  private handleInput() {
    const now = Date.now();
    if (now - this.lastKeyPressTime < this.keyRepeatDelay) return;

    // 前進
    if (Phaser.Input.Keyboard.JustDown(this.cursors.up) || Phaser.Input.Keyboard.JustDown(this.wasd.up)) {
      this.lastKeyPressTime = now;
      this.moveForward();
    }
    // 後退
    else if (Phaser.Input.Keyboard.JustDown(this.cursors.down) || Phaser.Input.Keyboard.JustDown(this.wasd.down)) {
      this.lastKeyPressTime = now;
      this.moveBackward();
    }
    // 左回転
    else if (Phaser.Input.Keyboard.JustDown(this.cursors.left) || Phaser.Input.Keyboard.JustDown(this.wasd.left)) {
      this.lastKeyPressTime = now;
      this.turnLeft();
    }
    // 右回転
    else if (Phaser.Input.Keyboard.JustDown(this.cursors.right) || Phaser.Input.Keyboard.JustDown(this.wasd.right)) {
      this.lastKeyPressTime = now;
      this.turnRight();
    }
  }

  private moveForward() {
    const angle = this.getAngle();
    const newX = this.playerX + Math.round(Math.cos(angle));
    const newY = this.playerY + Math.round(Math.sin(angle));

    if (this.canMoveTo(newX, newY)) {
      this.playerX = newX;
      this.playerY = newY;
      this.checkGoal();
    }
  }

  private moveBackward() {
    const angle = this.getAngle();
    const newX = this.playerX - Math.round(Math.cos(angle));
    const newY = this.playerY - Math.round(Math.sin(angle));

    if (this.canMoveTo(newX, newY)) {
      this.playerX = newX;
      this.playerY = newY;
      this.checkGoal();
    }
  }

  private turnLeft() {
    this.playerDir = (this.playerDir + 3) % 4; // -90度 = +270度
  }

  private turnRight() {
    this.playerDir = (this.playerDir + 1) % 4; // +90度
  }

  private getAngle(): number {
    // 0=東(0), 1=南(π/2), 2=西(π), 3=北(3π/2)
    return (this.playerDir * Math.PI) / 2;
  }

  private canMoveTo(x: number, y: number): boolean {
    if (y < 0 || y >= this.maze.length || x < 0 || x >= this.maze[0].length) {
      return false;
    }
    return this.maze[y][x] !== 1; // 壁でなければ移動可能
  }

  private checkGoal() {
    if (this.maze[this.playerY][this.playerX] === 2) {
      this.showGoalMessage();
    }
  }

  private checkNearbySign() {
    // プレイヤーの近くに看板があるかチェック
    let foundSign = false;

    this.signs.forEach((message, key) => {
      const [x, y] = key.split(',').map(Number);
      const dx = x - this.playerX;
      const dy = y - this.playerY;
      const distance = Math.sqrt(dx * dx + dy * dy);

      if (distance < this.signDisplayDistance) {
        this.currentSignMessage = message;
        this.signText.setText(message);
        this.signText.setVisible(true);
        foundSign = true;
      }
    });

    if (!foundSign) {
      this.signText.setVisible(false);
      this.currentSignMessage = '';
    }
  }

  private render3DView() {
    this.graphics.clear();

    const width = this.scale.width;
    const height = this.scale.height;
    const stripWidth = width / this.NUM_RAYS;

    // 空と地面の描画
    this.graphics.fillStyle(0x87CEEB, 1); // 空色
    this.graphics.fillRect(0, 0, width, height / 2);
    this.graphics.fillStyle(0x4a3c28, 1); // 茶色の地面
    this.graphics.fillRect(0, height / 2, width, height / 2);

    const playerAngle = this.getAngle();
    const playerPosX = this.playerX + 0.5; // グリッドの中心
    const playerPosY = this.playerY + 0.5; // グリッドの中心

    // 看板の描画用データを収集
    const signsToDraw: Array<{ angle: number; distance: number; message: string }> = [];

    this.signs.forEach((message, key) => {
      const [sx, sy] = key.split(',').map(Number);
      const dx = sx + 0.5 - playerPosX;
      const dy = sy + 0.5 - playerPosY;
      const distance = Math.sqrt(dx * dx + dy * dy);
      const angle = Math.atan2(dy, dx);

      if (distance < this.MAX_DEPTH) {
        signsToDraw.push({ angle, distance, message });
      }
    });

    // レイキャスティング
    for (let i = 0; i < this.NUM_RAYS; i++) {
      const rayAngle = playerAngle - this.FOV / 2 + (this.FOV * i) / this.NUM_RAYS;
      const { distance, hitSide } = this.castRay(rayAngle, playerPosX, playerPosY);

      // 魚眼補正
      const correctedDistance = distance * Math.cos(rayAngle - playerAngle);

      // 壁の高さ計算
      const wallHeight = correctedDistance > 0 ? (height / correctedDistance) * 0.5 : height;
      const wallTop = (height - wallHeight) / 2;

      // 距離による明度調整
      const brightness = Math.max(0, 1 - correctedDistance / this.MAX_DEPTH);

      // トウモロコシ畑の色（緑がかった黄色）
      let color: number;
      if (hitSide === 'horizontal') {
        color = Phaser.Display.Color.GetColor(
          Math.floor(204 * brightness),
          Math.floor(153 * brightness),
          Math.floor(51 * brightness)
        );
      } else {
        color = Phaser.Display.Color.GetColor(
          Math.floor(153 * brightness),
          Math.floor(102 * brightness),
          Math.floor(0 * brightness)
        );
      }

      this.graphics.fillStyle(color, 1);
      this.graphics.fillRect(i * stripWidth, wallTop, stripWidth + 1, wallHeight);
    }

    // 看板を描画（壁の手前に表示）
    signsToDraw.forEach((sign) => {
      const angleDiff = sign.angle - playerAngle;
      const normalizedAngle = Math.atan2(Math.sin(angleDiff), Math.cos(angleDiff));

      // FOV内にあるかチェック
      if (Math.abs(normalizedAngle) < this.FOV / 2) {
        const correctedDistance = sign.distance * Math.cos(normalizedAngle);
        const signHeight = correctedDistance > 0 ? (height / correctedDistance) * 0.3 : height * 0.3;
        const signWidth = signHeight * 1.5;
        const signTop = (height - signHeight) / 2;

        // 画面上のX位置を計算
        const screenX = ((normalizedAngle + this.FOV / 2) / this.FOV) * width - signWidth / 2;

        // 看板の背景（茶色の板）
        this.graphics.fillStyle(0x8B4513, 0.9);
        this.graphics.fillRect(screenX, signTop, signWidth, signHeight);

        // 看板の枠
        this.graphics.lineStyle(3, 0x000000, 1);
        this.graphics.strokeRect(screenX, signTop, signWidth, signHeight);

        // 簡易的なテキスト描画（"！"マーク）
        this.graphics.fillStyle(0xFF0000, 1);
        const exclamationWidth = signWidth * 0.3;
        const exclamationHeight = signHeight * 0.5;
        const exclamationX = screenX + signWidth / 2 - exclamationWidth / 2;
        const exclamationY = signTop + signHeight * 0.2;

        // ビックリマーク
        this.graphics.fillRect(
          exclamationX,
          exclamationY,
          exclamationWidth,
          exclamationHeight * 0.7
        );
        this.graphics.fillCircle(
          exclamationX + exclamationWidth / 2,
          exclamationY + exclamationHeight,
          exclamationWidth / 2
        );
      }
    });
  }

  private castRay(
    angle: number,
    playerX: number,
    playerY: number
  ): { distance: number; hitSide: 'horizontal' | 'vertical' } {
    const rayDirX = Math.cos(angle);
    const rayDirY = Math.sin(angle);

    let mapX = Math.floor(playerX);
    let mapY = Math.floor(playerY);

    const deltaDistX = Math.abs(1 / rayDirX);
    const deltaDistY = Math.abs(1 / rayDirY);

    let stepX: number;
    let stepY: number;
    let sideDistX: number;
    let sideDistY: number;

    if (rayDirX < 0) {
      stepX = -1;
      sideDistX = (playerX - mapX) * deltaDistX;
    } else {
      stepX = 1;
      sideDistX = (mapX + 1.0 - playerX) * deltaDistX;
    }

    if (rayDirY < 0) {
      stepY = -1;
      sideDistY = (playerY - mapY) * deltaDistY;
    } else {
      stepY = 1;
      sideDistY = (mapY + 1.0 - playerY) * deltaDistY;
    }

    let hit = false;
    let side: 'horizontal' | 'vertical' = 'vertical';

    while (!hit && mapX >= 0 && mapY >= 0 && mapY < this.maze.length && mapX < this.maze[0].length) {
      if (sideDistX < sideDistY) {
        sideDistX += deltaDistX;
        mapX += stepX;
        side = 'vertical';
      } else {
        sideDistY += deltaDistY;
        mapY += stepY;
        side = 'horizontal';
      }

      if (mapY >= 0 && mapY < this.maze.length && mapX >= 0 && mapX < this.maze[0].length) {
        if (this.maze[mapY][mapX] === 1) {
          hit = true;
        }
      } else {
        hit = true;
      }
    }

    let distance: number;
    if (side === 'vertical') {
      distance = Math.abs((mapX - playerX + (1 - stepX) / 2) / rayDirX);
    } else {
      distance = Math.abs((mapY - playerY + (1 - stepY) / 2) / rayDirY);
    }

    return { distance, hitSide: side };
  }

  private renderMinimap() {
    this.minimapGraphics.clear();

    const minimapSize = Math.min(this.scale.width, this.scale.height) * 0.25;
    const minimapX = this.scale.width - minimapSize - 10;
    const minimapY = this.scale.height - minimapSize - 10;
    const cellSize = minimapSize / Math.max(this.maze.length, this.maze[0].length);

    // 半透明背景
    this.minimapGraphics.fillStyle(0x000000, 0.5);
    this.minimapGraphics.fillRect(minimapX - 5, minimapY - 5, minimapSize + 10, minimapSize + 10);

    // 迷路の描画（常に北が上）
    for (let y = 0; y < this.maze.length; y++) {
      for (let x = 0; x < this.maze[y].length; x++) {
        if (this.maze[y][x] === 1) {
          this.minimapGraphics.fillStyle(0x228B22, 0.8); // 壁は緑
        } else if (this.maze[y][x] === 2) {
          this.minimapGraphics.fillStyle(0xFFD700, 0.8); // ゴールは金色
        } else if (this.maze[y][x] === 3) {
          this.minimapGraphics.fillStyle(0xFF4444, 0.8); // 看板は赤
        } else {
          this.minimapGraphics.fillStyle(0xFFFFFF, 0.3); // 通路は白
        }
        this.minimapGraphics.fillRect(
          minimapX + x * cellSize,
          minimapY + y * cellSize,
          cellSize,
          cellSize
        );
      }
    }

    // プレイヤーの描画（グリッド中央）
    const playerScreenX = minimapX + (this.playerX + 0.5) * cellSize;
    const playerScreenY = minimapY + (this.playerY + 0.5) * cellSize;
    this.minimapGraphics.fillStyle(0xFF0000, 1);
    this.minimapGraphics.fillCircle(playerScreenX, playerScreenY, cellSize / 2);

    // プレイヤーの向き（常に北が上の座標系）
    const playerAngle = this.getAngle();
    const dirLength = cellSize;
    const dirEndX = playerScreenX + Math.cos(playerAngle) * dirLength;
    const dirEndY = playerScreenY + Math.sin(playerAngle) * dirLength;
    this.minimapGraphics.lineStyle(2, 0xFF0000, 1);
    this.minimapGraphics.lineBetween(playerScreenX, playerScreenY, dirEndX, dirEndY);
  }

  private showGoalMessage() {
    const text = this.add.text(
      this.scale.width / 2,
      this.scale.height / 2,
      'トウモロコシ畑から脱出成功！',
      {
        fontSize: '48px',
        color: '#FFD700',
        backgroundColor: '#000000',
        padding: { x: 20, y: 10 },
      }
    );
    text.setOrigin(0.5);

    // ゲームを一時停止
    this.scene.pause();
  }
}
