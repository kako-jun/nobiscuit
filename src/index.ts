import Phaser from 'phaser';
import MazeScene from './scenes/MazeScene';

const config: Phaser.Types.Core.GameConfig = {
  type: Phaser.AUTO,
  width: window.innerWidth,
  height: window.innerHeight,
  parent: 'game-container',
  backgroundColor: '#000000',
  scale: {
    mode: Phaser.Scale.RESIZE,
    autoCenter: Phaser.Scale.CENTER_BOTH,
  },
  scene: [MazeScene],
};

const game = new Phaser.Game(config);

// リサイズ対応
window.addEventListener('resize', () => {
  game.scale.resize(window.innerWidth, window.innerHeight);
});
