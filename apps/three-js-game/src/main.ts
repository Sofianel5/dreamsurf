import * as THREE from 'three';
import { HillyGame } from '@/games/HillyGame';
import { GLTFGame } from '@/games/GLTFGame';
import { MazeGame } from '@/games/MazeGame';
import { HomeScreen } from '@/ui/HomeScreen';

// Initialize home screen
const homeScreen = new HomeScreen();
let game: HillyGame | null = null;
let gltfGame: GLTFGame | null = null;
let mazeGame: MazeGame | null = null;
let currentGame: 'hilly' | 'gltf' | 'maze' | null = null;

// Show home screen initially
homeScreen.showHome();

// Global functions for HTML onclick handlers
(window as any).startGame = () => {
  const settings = homeScreen.getSettings();

  // Stop other games if running
  if (gltfGame) gltfGame.pause();
  if (mazeGame) mazeGame.pause();

  // Initialize game with settings if not already created
  if (!game) {
    game = new HillyGame(settings);
  } else {
    // Update game settings
    game.updateSettings(settings);
  }

  currentGame = 'hilly';
  homeScreen.showGame();

  // Start the game loop
  game.start();
};

(window as any).startGltfGame = () => {
  const settings = homeScreen.getSettings();

  // Stop other games if running
  if (game) game.pause();
  if (mazeGame) mazeGame.pause();

  // Initialize GLTF game with settings if not already created
  if (!gltfGame) {
    gltfGame = new GLTFGame(settings);
  } else {
    // Update game settings
    gltfGame.updateSettings(settings);
  }

  currentGame = 'gltf';
  homeScreen.showGame();

  // Start the GLTF game loop
  gltfGame.start();
};

(window as any).startMazeGame = () => {
  const settings = homeScreen.getSettings();

  // Stop other games if running
  if (game) game.pause();
  if (gltfGame) gltfGame.pause();

  // Initialize maze game with settings if not already created
  if (!mazeGame) {
    mazeGame = new MazeGame(settings);
  } else {
    // Update game settings
    mazeGame.updateSettings(settings);
  }

  currentGame = 'maze';
  homeScreen.showGame();

  // Start the maze game loop
  mazeGame.start();
};

(window as any).showSettings = () => {
  homeScreen.showSettings();
};

(window as any).backToHome = () => {
  homeScreen.showHome();

  // Pause/stop the games if running
  if (game) game.pause();
  if (gltfGame) gltfGame.pause();
  if (mazeGame) mazeGame.pause();

  currentGame = null;
};