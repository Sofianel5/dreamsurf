import React, { useState } from 'react';
import { HomeScreen } from '@/components/HomeScreen';
import { GameScreen } from '@/components/GameScreen';
import { SettingsScreen } from '@/components/SettingsScreen';
import { HillyGame } from '@/games/HillyGame';
import { GLTFGame } from '@/games/GLTFGame';
import { MazeGame } from '@/games/MazeGame';
import './App.css';

export interface GameSettings {
  moveSpeed: number;
  mouseSensitivity: number;
  graphicsQuality: 'low' | 'medium' | 'high';
}

type Screen = 'home' | 'settings' | 'game';
type GameType = 'hilly' | 'gltf' | 'maze' | null;

export default function App() {
  const [currentScreen, setCurrentScreen] = useState<Screen>('home');
  const [currentGame, setCurrentGame] = useState<GameType>(null);
  const [settings, setSettings] = useState<GameSettings>({
    moveSpeed: 1000,
    mouseSensitivity: 0.002,
    graphicsQuality: 'medium'
  });

  // Game instances
  const [hillyGame, setHillyGame] = useState<HillyGame | null>(null);
  const [gltfGame, setGltfGame] = useState<GLTFGame | null>(null);
  const [mazeGame, setMazeGame] = useState<MazeGame | null>(null);

  const startGame = (gameType: GameType) => {
    // Pause other games
    if (hillyGame) hillyGame.pause();
    if (gltfGame) gltfGame.pause();
    if (mazeGame) mazeGame.pause();

    setCurrentGame(gameType);
    setCurrentScreen('game');

    // Start the selected game
    if (gameType === 'hilly') {
      if (!hillyGame) {
        const game = new HillyGame(settings);
        setHillyGame(game);
        setTimeout(() => game.start(), 100); // Allow React to render first
      } else {
        hillyGame.updateSettings(settings);
        setTimeout(() => hillyGame.start(), 100);
      }
    } else if (gameType === 'gltf') {
      if (!gltfGame) {
        const game = new GLTFGame(settings);
        setGltfGame(game);
        setTimeout(() => game.start(), 100);
      } else {
        gltfGame.updateSettings(settings);
        setTimeout(() => gltfGame.start(), 100);
      }
    } else if (gameType === 'maze') {
      if (!mazeGame) {
        const game = new MazeGame(settings);
        setMazeGame(game);
        setTimeout(() => game.start(), 100);
      } else {
        mazeGame.updateSettings(settings);
        setTimeout(() => mazeGame.start(), 100);
      }
    }
  };

  const backToHome = () => {
    // Pause all games
    if (hillyGame) hillyGame.pause();
    if (gltfGame) gltfGame.pause();
    if (mazeGame) mazeGame.pause();

    setCurrentGame(null);
    setCurrentScreen('home');
  };

  const showSettings = () => {
    setCurrentScreen('settings');
  };

  const updateSettings = (newSettings: GameSettings) => {
    setSettings(newSettings);

    // Update active game settings
    if (currentGame === 'hilly' && hillyGame) {
      hillyGame.updateSettings(newSettings);
    } else if (currentGame === 'gltf' && gltfGame) {
      gltfGame.updateSettings(newSettings);
    } else if (currentGame === 'maze' && mazeGame) {
      mazeGame.updateSettings(newSettings);
    }
  };

  return (
    <div className="app">
      {currentScreen === 'home' && (
        <HomeScreen
          onStartGame={(gameType) => startGame(gameType)}
          onShowSettings={showSettings}
        />
      )}

      {currentScreen === 'settings' && (
        <SettingsScreen
          settings={settings}
          onUpdateSettings={updateSettings}
          onBack={backToHome}
        />
      )}

      {currentScreen === 'game' && (
        <GameScreen
          onBack={backToHome}
          gameRenderer={
            currentGame === 'hilly' ? hillyGame?.getRenderer() :
            currentGame === 'gltf' ? gltfGame?.getRenderer() :
            currentGame === 'maze' ? mazeGame?.getRenderer() : undefined
          }
        />
      )}
    </div>
  );
}