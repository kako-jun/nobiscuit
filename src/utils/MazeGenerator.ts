// 迷路生成ユーティリティ

export interface MazeCell {
  type: 'wall' | 'path' | 'goal' | 'sign';
  message?: string; // 看板用メッセージ
}

export class MazeGenerator {
  private width: number;
  private height: number;
  private maze: number[][];
  private signs: Map<string, string>; // 看板の位置とメッセージ

  // 煽りメッセージ集
  private readonly TAUNT_MESSAGES = [
    'ばーか！ここは行き止まりだよ！',
    'お前、方向音痴か？',
    'また迷ったの？www',
    'ざまぁwww',
    'センスないねぇ〜',
    '才能ないから諦めろ',
    'GPS使えよ無能',
    'ここまで来て引き返すの？ださっ',
    'お疲れ様でした（笑）',
    '脳みそついてる？',
    'お前の時間、無駄になったな',
    'マジで下手くそで草',
    '小学生でももっと上手いぞ',
    '迷路苦手？幼稚園からやり直せ',
    'ここはゴールじゃないよバーカ',
  ];

  constructor(width: number = 16, height: number = 13) {
    this.width = width;
    this.height = height;
    this.maze = [];
    this.signs = new Map();
  }

  public generate(): { maze: number[][]; signs: Map<string, string> } {
    this.initializeMaze();
    this.generateMazeDFS(1, 1);
    this.setGoal();
    this.placeSignsAtDeadEnds();
    return { maze: this.maze, signs: this.signs };
  }

  private initializeMaze(): void {
    this.maze = [];
    for (let y = 0; y < this.height; y++) {
      this.maze[y] = [];
      for (let x = 0; x < this.width; x++) {
        this.maze[y][x] = 1; // すべて壁で初期化
      }
    }
  }

  private generateMazeDFS(x: number, y: number): void {
    this.maze[y][x] = 0; // 現在位置を通路にする

    // 4方向をランダムな順序で探索
    const directions = this.shuffleArray([
      { dx: 0, dy: -2 }, // 上
      { dx: 2, dy: 0 },  // 右
      { dx: 0, dy: 2 },  // 下
      { dx: -2, dy: 0 }, // 左
    ]);

    for (const dir of directions) {
      const nx = x + dir.dx;
      const ny = y + dir.dy;

      if (this.isValidCell(nx, ny) && this.maze[ny][nx] === 1) {
        // 間の壁を削除
        this.maze[y + dir.dy / 2][x + dir.dx / 2] = 0;
        this.generateMazeDFS(nx, ny);
      }
    }
  }

  private isValidCell(x: number, y: number): boolean {
    return x > 0 && x < this.width - 1 && y > 0 && y < this.height - 1;
  }

  private shuffleArray<T>(array: T[]): T[] {
    const result = [...array];
    for (let i = result.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [result[i], result[j]] = [result[j], result[i]];
    }
    return result;
  }

  private setGoal(): void {
    // 右下あたりにゴールを配置
    for (let y = this.height - 2; y > 0; y--) {
      for (let x = this.width - 2; x > 0; x--) {
        if (this.maze[y][x] === 0) {
          this.maze[y][x] = 2; // ゴール
          return;
        }
      }
    }
  }

  private placeSignsAtDeadEnds(): void {
    for (let y = 1; y < this.height - 1; y++) {
      for (let x = 1; x < this.width - 1; x++) {
        if (this.maze[y][x] === 0 && this.isDeadEnd(x, y)) {
          // 行き止まりに看板を配置
          const message = this.getRandomTauntMessage();
          this.signs.set(`${x},${y}`, message);
          this.maze[y][x] = 3; // 3 = 看板がある通路
        }
      }
    }
  }

  private isDeadEnd(x: number, y: number): boolean {
    // 隣接する通路（0,2,3）が1つ以下なら行き止まり
    const directions = [
      { dx: 0, dy: -1 }, // 上
      { dx: 1, dy: 0 },  // 右
      { dx: 0, dy: 1 },  // 下
      { dx: -1, dy: 0 }, // 左
    ];

    let openCount = 0;
    for (const dir of directions) {
      const nx = x + dir.dx;
      const ny = y + dir.dy;
      if (
        nx >= 0 &&
        nx < this.width &&
        ny >= 0 &&
        ny < this.height &&
        (this.maze[ny][nx] === 0 || this.maze[ny][nx] === 2 || this.maze[ny][nx] === 3)
      ) {
        openCount++;
      }
    }

    return openCount <= 1;
  }

  private getRandomTauntMessage(): string {
    return this.TAUNT_MESSAGES[Math.floor(Math.random() * this.TAUNT_MESSAGES.length)];
  }
}
