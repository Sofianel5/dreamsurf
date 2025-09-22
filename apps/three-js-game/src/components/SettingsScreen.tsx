import React, { useState, useEffect } from 'react';
import { GameSettings } from '@/App';

interface SettingsScreenProps {
  settings: GameSettings;
  onUpdateSettings: (settings: GameSettings) => void;
  onBack: () => void;
}

export function SettingsScreen({ settings, onUpdateSettings, onBack }: SettingsScreenProps) {
  const [localSettings, setLocalSettings] = useState<GameSettings>(settings);

  useEffect(() => {
    setLocalSettings(settings);
  }, [settings]);

  const handleSettingChange = (key: keyof GameSettings, value: string | number) => {
    const newSettings = { ...localSettings, [key]: value };
    setLocalSettings(newSettings);
    onUpdateSettings(newSettings);
  };

  return (
    <div className="settings-screen">
      <h2 className="settings-title">Settings</h2>

      <div className="settings-option">
        <label htmlFor="moveSpeed">Move Speed:</label>
        <input
          type="range"
          id="moveSpeed"
          min={100}
          max={2000}
          value={localSettings.moveSpeed}
          onChange={(e) => handleSettingChange('moveSpeed', parseInt(e.target.value))}
        />
        <span className="settings-value">{localSettings.moveSpeed}</span>
      </div>

      <div className="settings-option">
        <label htmlFor="mouseSensitivity">Mouse Sensitivity:</label>
        <input
          type="range"
          id="mouseSensitivity"
          min={0.001}
          max={0.01}
          step={0.001}
          value={localSettings.mouseSensitivity}
          onChange={(e) => handleSettingChange('mouseSensitivity', parseFloat(e.target.value))}
        />
        <span className="settings-value">{localSettings.mouseSensitivity.toFixed(3)}</span>
      </div>

      <div className="settings-option">
        <label htmlFor="graphicsQuality">Graphics Quality:</label>
        <select
          id="graphicsQuality"
          value={localSettings.graphicsQuality}
          onChange={(e) => handleSettingChange('graphicsQuality', e.target.value as 'low' | 'medium' | 'high')}
        >
          <option value="low">Low</option>
          <option value="medium">Medium</option>
          <option value="high">High</option>
        </select>
      </div>

      <button className="menu-button" onClick={onBack}>
        Back
      </button>
    </div>
  );
}