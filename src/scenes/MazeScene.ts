import Phaser from 'phaser';

export default class MazeScene extends Phaser.Scene {
  // 迷路データ (1=壁, 0=通路, 2=ゴール)
  private maze: number[][] = [
    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
    [1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 1, 0, 1, 0, 1, 1, 1, 1, 1, 0, 1, 1, 0, 1],
    [1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 1],
    [1, 0, 1, 1, 1, 1, 1, 0, 1, 0, 1, 1, 0, 1, 0, 1],
    [1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 1, 0, 1],
    [1, 1, 1, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 1, 0, 1],
    [1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1],
    [1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 0, 1],
    [1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1],
    [1, 0, 1, 1, 1, 0, 1, 1, 0, 1, 1, 1, 1, 1, 0, 1],
    [1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 1],
    [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
  ];

  // プレイヤー位置と向き
  private playerX: number = 1.5;
  private playerY: number = 1.5;
  private playerAngle: number = 0; // ラジアン

  // レイキャスティング設定
  private readonly FOV = Math.PI / 3; // 視野角60度
  private readonly NUM_RAYS = 120; // レイ数（解像度）
  private readonly MAX_DEPTH = 20; // 最大描画距離

  // グラフィックス
  private graphics!: Phaser.GameObjects.Graphics;
  private minimapGraphics!: Phaser.GameObjects.Graphics;

  // 入力
  private cursors!: Phaser.Types.Input.Keyboard.CursorKeys;
  private wasd: any = {};

  constructor() {
    super({ key: 'MazeScene' });
  }

  create() {
    this.graphics = this.add.graphics();
    this.minimapGraphics = this.add.graphics();

    // キーボード入力
    this.cursors = this.input.keyboard!.createCursorKeys();
    this.wasd = {
      up: this.input.keyboard!.addKey(Phaser.Input.Keyboard.KeyCodes.W),
      down: this.input.keyboard!.addKey(Phaser.Input.Keyboard.KeyCodes.S),
      left: this.input.keyboard!.addKey(Phaser.Input.Keyboard.KeyCodes.A),
      right: this.input.keyboard!.addKey(Phaser.Input.Keyboard.KeyCodes.D),
    };

    // タッチ操作（簡易実装：後で改善）
    this.input.on('pointerdown', (pointer: Phaser.Input.Pointer) => {
      const centerX = this.scale.width / 2;
      const centerY = this.scale.height / 2;

      // 画面を4分割：上=前進、下=後退、左=左回転、右=右回転
      if (pointer.y < centerY) {
        this.movePlayer(0.1);
      } else {
        this.movePlayer(-0.1);
      }

      if (pointer.x < centerX) {
        this.playerAngle -= 0.1;
      } else {
        this.playerAngle += 0.1;
      }
    });
  }

  update(time: number, delta: number) {
    this.handleInput(delta);
    this.render3DView();
    this.renderMinimap();
  }

  private handleInput(delta: number) {
    const moveSpeed = 2 / 1000; // マス/ms
    const rotateSpeed = 2 / 1000; // ラジアン/ms

    // 前進・後退
    if (this.cursors.up.isDown || this.wasd.up.isDown) {
      this.movePlayer(moveSpeed * delta);
    }
    if (this.cursors.down.isDown || this.wasd.down.isDown) {
      this.movePlayer(-moveSpeed * delta);
    }

    // 回転
    if (this.cursors.left.isDown || this.wasd.left.isDown) {
      this.playerAngle -= rotateSpeed * delta;
    }
    if (this.cursors.right.isDown || this.wasd.right.isDown) {
      this.playerAngle += rotateSpeed * delta;
    }
  }

  private movePlayer(distance: number) {
    const newX = this.playerX + Math.cos(this.playerAngle) * distance;
    const newY = this.playerY + Math.sin(this.playerAngle) * distance;

    // 壁判定
    if (this.maze[Math.floor(newY)][Math.floor(newX)] !== 1) {
      this.playerX = newX;
      this.playerY = newY;

      // ゴール判定
      if (this.maze[Math.floor(newY)][Math.floor(newX)] === 2) {
        this.showGoalMessage();
      }
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

    // レイキャスティング
    for (let i = 0; i < this.NUM_RAYS; i++) {
      const rayAngle = this.playerAngle - this.FOV / 2 + (this.FOV * i) / this.NUM_RAYS;
      const { distance, hitSide } = this.castRay(rayAngle);

      // 魚眼補正
      const correctedDistance = distance * Math.cos(rayAngle - this.playerAngle);

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
  }

  private castRay(angle: number): { distance: number; hitSide: 'horizontal' | 'vertical' } {
    const rayDirX = Math.cos(angle);
    const rayDirY = Math.sin(angle);

    let mapX = Math.floor(this.playerX);
    let mapY = Math.floor(this.playerY);

    const deltaDistX = Math.abs(1 / rayDirX);
    const deltaDistY = Math.abs(1 / rayDirY);

    let stepX: number;
    let stepY: number;
    let sideDistX: number;
    let sideDistY: number;

    if (rayDirX < 0) {
      stepX = -1;
      sideDistX = (this.playerX - mapX) * deltaDistX;
    } else {
      stepX = 1;
      sideDistX = (mapX + 1.0 - this.playerX) * deltaDistX;
    }

    if (rayDirY < 0) {
      stepY = -1;
      sideDistY = (this.playerY - mapY) * deltaDistY;
    } else {
      stepY = 1;
      sideDistY = (mapY + 1.0 - this.playerY) * deltaDistY;
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
      distance = Math.abs((mapX - this.playerX + (1 - stepX) / 2) / rayDirX);
    } else {
      distance = Math.abs((mapY - this.playerY + (1 - stepY) / 2) / rayDirY);
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

    // 迷路の描画
    for (let y = 0; y < this.maze.length; y++) {
      for (let x = 0; x < this.maze[y].length; x++) {
        if (this.maze[y][x] === 1) {
          this.minimapGraphics.fillStyle(0x228B22, 0.8); // 壁は緑
        } else if (this.maze[y][x] === 2) {
          this.minimapGraphics.fillStyle(0xFFD700, 0.8); // ゴールは金色
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

    // プレイヤーの描画
    const playerScreenX = minimapX + this.playerX * cellSize;
    const playerScreenY = minimapY + this.playerY * cellSize;
    this.minimapGraphics.fillStyle(0xFF0000, 1);
    this.minimapGraphics.fillCircle(playerScreenX, playerScreenY, cellSize / 2);

    // プレイヤーの向き
    const dirLength = cellSize;
    const dirEndX = playerScreenX + Math.cos(this.playerAngle) * dirLength;
    const dirEndY = playerScreenY + Math.sin(this.playerAngle) * dirLength;
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
