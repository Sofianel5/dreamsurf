import * as THREE from 'three';
import { Game } from '@/game';
import { HomeScreen } from '@/ui/HomeScreen';

// Initialize home screen
const homeScreen = new HomeScreen();
let game: Game | null = null;

// Show home screen initially
homeScreen.showHome();

// Global functions for HTML onclick handlers
(window as any).startGame = () => {
  const settings = homeScreen.getSettings();

  // Initialize game with settings if not already created
  if (!game) {
    game = new Game(settings);
  } else {
    // Update game settings
    game.updateSettings(settings);
  }

  homeScreen.showGame();

  // Start the game loop
  game.start();
};

(window as any).showSettings = () => {
  homeScreen.showSettings();
};

(window as any).backToHome = () => {
  homeScreen.showHome();

  // Pause/stop the game if running
  if (game) {
    game.pause();
  }
};