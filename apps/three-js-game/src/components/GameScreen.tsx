import React, { useRef, useEffect } from 'react';
import * as THREE from 'three';

interface GameScreenProps {
  onBack: () => void;
  gameRenderer?: THREE.WebGLRenderer;
}

export function GameScreen({ onBack, gameRenderer }: GameScreenProps) {
  const gameContainerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (gameRenderer && gameContainerRef.current) {
      // Clear any existing children
      gameContainerRef.current.innerHTML = '';
      // Append the Three.js canvas
      gameContainerRef.current.appendChild(gameRenderer.domElement);
    }
  }, [gameRenderer]);

  return (
    <div className="game-screen">
      <div ref={gameContainerRef} style={{ width: '100%', height: '100%' }} />
      <div className="game-info">
        WASD/Arrow Keys - Move<br/>
        Mouse - Look Around<br/>
        Space - Jump
      </div>
      <button className="back-button" onClick={onBack}>
        Back to Menu
      </button>
    </div>
  );
}