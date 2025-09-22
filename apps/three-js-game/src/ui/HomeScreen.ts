export class HomeScreen {
  private homeScreen: HTMLElement;
  private settingsScreen: HTMLElement;
  private gameScreen: HTMLElement;
  private settings: GameSettings;

  constructor() {
    this.homeScreen = document.getElementById('homeScreen')!;
    this.settingsScreen = document.getElementById('settingsScreen')!;
    this.gameScreen = document.getElementById('gameScreen')!;

    this.settings = {
      moveSpeed: 1000,
      mouseSensitivity: 0.002,
      graphicsQuality: 'medium'
    };

    this.initializeSettingsControls();
    this.loadSettings();
  }

  private initializeSettingsControls(): void {
    // Move Speed slider
    const moveSpeedSlider = document.getElementById('moveSpeed') as HTMLInputElement;
    const moveSpeedValue = document.getElementById('moveSpeedValue')!;

    moveSpeedSlider.addEventListener('input', (e) => {
      const value = (e.target as HTMLInputElement).value;
      moveSpeedValue.textContent = value;
      this.settings.moveSpeed = parseInt(value);
      this.saveSettings();
    });

    // Mouse Sensitivity slider
    const mouseSensitivitySlider = document.getElementById('mouseSensitivity') as HTMLInputElement;
    const mouseSensitivityValue = document.getElementById('mouseSensitivityValue')!;

    mouseSensitivitySlider.addEventListener('input', (e) => {
      const value = (e.target as HTMLInputElement).value;
      mouseSensitivityValue.textContent = value;
      this.settings.mouseSensitivity = parseFloat(value) / 100; // Convert to game scale
      this.saveSettings();
    });

    // Graphics Quality dropdown
    const graphicsSelect = document.getElementById('graphicsQuality') as HTMLSelectElement;
    graphicsSelect.addEventListener('change', (e) => {
      this.settings.graphicsQuality = (e.target as HTMLSelectElement).value;
      this.saveSettings();
    });
  }

  public showHome(): void {
    this.homeScreen.style.display = 'flex';
    this.settingsScreen.style.display = 'none';
    this.gameScreen.style.display = 'none';
  }

  public showSettings(): void {
    this.homeScreen.style.display = 'none';
    this.settingsScreen.style.display = 'flex';
    this.gameScreen.style.display = 'none';
    this.updateSettingsDisplay();
  }

  public showGame(): void {
    this.homeScreen.style.display = 'none';
    this.settingsScreen.style.display = 'none';
    this.gameScreen.style.display = 'block';
  }

  private updateSettingsDisplay(): void {
    (document.getElementById('moveSpeed') as HTMLInputElement).value = this.settings.moveSpeed.toString();
    document.getElementById('moveSpeedValue')!.textContent = this.settings.moveSpeed.toString();

    (document.getElementById('mouseSensitivity') as HTMLInputElement).value = (this.settings.mouseSensitivity * 100).toString();
    document.getElementById('mouseSensitivityValue')!.textContent = (this.settings.mouseSensitivity * 100).toString();

    (document.getElementById('graphicsQuality') as HTMLSelectElement).value = this.settings.graphicsQuality;
  }

  public getSettings(): GameSettings {
    return { ...this.settings };
  }

  private saveSettings(): void {
    localStorage.setItem('hillyWorldSettings', JSON.stringify(this.settings));
  }

  private loadSettings(): void {
    const saved = localStorage.getItem('hillyWorldSettings');
    if (saved) {
      this.settings = { ...this.settings, ...JSON.parse(saved) };
    }
  }
}

export interface GameSettings {
  moveSpeed: number;
  mouseSensitivity: number;
  graphicsQuality: string;
}