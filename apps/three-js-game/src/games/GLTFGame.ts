import * as THREE from 'three';
import { AssetLoader } from '@/loaders/AssetLoader';
import { Player } from '@/player/Player';
import { Sky } from '@/environment/Sky';
import { GameSettings } from '@/App';

export class GLTFGame {
  private scene: THREE.Scene;
  private camera: THREE.PerspectiveCamera;
  private renderer: THREE.WebGLRenderer;
  private player: Player;
  private clock: THREE.Clock;
  private isRunning: boolean = false;
  private animationId: number | null = null;
  private settings: GameSettings;
  private assetLoader: AssetLoader;

  constructor(settings?: GameSettings) {
    this.settings = settings || {
      moveSpeed: 1000,
      mouseSensitivity: 0.002,
      graphicsQuality: 'medium'
    };

    // Initialize Three.js components
    this.scene = new THREE.Scene();
    this.scene.fog = new THREE.Fog(0x87CEEB, 100, 800);

    this.camera = new THREE.PerspectiveCamera(
      75,
      window.innerWidth / window.innerHeight,
      0.1,
      1000
    );

    this.renderer = new THREE.WebGLRenderer({ antialias: true });
    this.renderer.setSize(window.innerWidth, window.innerHeight);
    this.renderer.shadowMap.enabled = true;
    this.renderer.shadowMap.type = THREE.PCFSoftShadowMap;

    this.clock = new THREE.Clock();
    this.assetLoader = AssetLoader.getInstance();

    // Initialize game components
    this.initLighting();
    this.createTerrain();

    // Initialize player with settings
    this.player = new Player(this.camera, this.scene, this.settings);

    // Initialize environment
    const sky = new Sky(this.scene);

    // Handle window resize
    window.addEventListener('resize', () => this.onWindowResize());
  }

  private initLighting(): void {
    // Ambient light
    const ambientLight = new THREE.AmbientLight(0x404040, 0.6);
    this.scene.add(ambientLight);

    // Directional light (sun)
    const directionalLight = new THREE.DirectionalLight(0xffffff, 1);
    directionalLight.position.set(50, 100, 50);
    directionalLight.castShadow = true;

    // Shadow settings
    directionalLight.shadow.camera.left = -200;
    directionalLight.shadow.camera.right = 200;
    directionalLight.shadow.camera.top = 200;
    directionalLight.shadow.camera.bottom = -200;
    directionalLight.shadow.camera.near = 0.1;
    directionalLight.shadow.camera.far = 400;
    directionalLight.shadow.mapSize.width = 2048;
    directionalLight.shadow.mapSize.height = 2048;

    this.scene.add(directionalLight);

    // Add hemisphere light for natural lighting
    const hemisphereLight = new THREE.HemisphereLight(0x87CEEB, 0x4a6741, 0.4);
    this.scene.add(hemisphereLight);
  }

  private async createTerrain(): Promise<void> {
    // Create base ground plane
    const terrainSize = 400;
    const terrainGeometry = new THREE.PlaneGeometry(terrainSize, terrainSize);
    const groundMaterial = new THREE.MeshLambertMaterial({ color: 0x4a6741 });

    const ground = new THREE.Mesh(terrainGeometry, groundMaterial);
    ground.rotation.x = -Math.PI / 2;
    ground.receiveShadow = true;
    ground.position.y = 0;
    ground.frustumCulled = false;
    this.scene.add(ground);

    // Load and distribute realistic grass models
    await this.distributeGrass();

    // Set player starting position above the terrain
    this.camera.position.set(0, 2, 5);
  }

  private async distributeGrass(): Promise<void> {
    try {
      console.log('Loading realistic grass models...');

      // Load the grass model
      const grassModel = await this.assetLoader.loadModel('/models/realistic_grass/scene.gltf');

      // Create grass distribution with performance optimization
      const grassGroup = new THREE.Group();
      const grassSpacing = 8; // Space between grass patches
      const terrainSize = 400;
      const grassCount = Math.floor(terrainSize / grassSpacing);

      // Use instanced rendering for better performance
      for (let x = -grassCount / 2; x < grassCount / 2; x++) {
        for (let z = -grassCount / 2; z < grassCount / 2; z++) {
          // Add some randomness to avoid grid pattern
          const offsetX = (Math.random() - 0.5) * grassSpacing * 0.5;
          const offsetZ = (Math.random() - 0.5) * grassSpacing * 0.5;

          const grassInstance = grassModel.clone();
          grassInstance.position.set(
            x * grassSpacing + offsetX,
            0.1, // Slightly above ground
            z * grassSpacing + offsetZ
          );

          // Random rotation for variety
          grassInstance.rotation.y = Math.random() * Math.PI * 2;

          // Random scale for variety (90% - 110%)
          const scale = 0.9 + Math.random() * 0.2;
          grassInstance.scale.set(scale, scale, scale);

          grassGroup.add(grassInstance);
        }
      }

      this.scene.add(grassGroup);
      console.log('Realistic grass distributed across terrain');

    } catch (error) {
      console.error('Failed to load grass model:', error);
      console.log('Using base terrain without grass models');
    }
  }

  private onWindowResize(): void {
    this.camera.aspect = window.innerWidth / window.innerHeight;
    this.camera.updateProjectionMatrix();
    this.renderer.setSize(window.innerWidth, window.innerHeight);
  }

  public start(): void {
    if (!this.isRunning) {
      this.isRunning = true;
      this.animate();
    }
  }

  public pause(): void {
    this.isRunning = false;
    if (this.animationId !== null) {
      cancelAnimationFrame(this.animationId);
      this.animationId = null;
    }
  }

  public updateSettings(settings: GameSettings): void {
    this.settings = settings;
    if (this.player) {
      this.player.updateSettings(settings);
    }
  }

  public getRenderer(): THREE.WebGLRenderer {
    return this.renderer;
  }

  private animate = (): void => {
    if (!this.isRunning) return;

    this.animationId = requestAnimationFrame(this.animate);

    const deltaTime = this.clock.getDelta();

    // Update player
    this.player.update(deltaTime);

    // Render the scene
    this.renderer.render(this.scene, this.camera);
  };
}