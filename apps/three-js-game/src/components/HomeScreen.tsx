import React from 'react';

interface HomeScreenProps {
  onStartGame: (gameType: 'procedural' | 'gltf' | 'maze') => void;
  onShowSettings: () => void;
}

export function HomeScreen({ onStartGame, onShowSettings }: HomeScreenProps) {
  return (
    <div className="home-screen">
      <h1 className="game-title">Hilly World</h1>
      <button className="menu-button" onClick={() => onStartGame('procedural')}>
        Play Procedural World
      </button>
      <button className="menu-button" onClick={() => onStartGame('gltf')}>
        Play GLTF
      </button>
      <button className="menu-button" onClick={() => onStartGame('maze')}>
        Play Maze
      </button>
      <button className="menu-button" onClick={onShowSettings}>
        Settings
      </button>
    </div>
  );
}