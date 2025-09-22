import * as THREE from 'three';
import { GLTFAssetLoader } from '@/loaders/GLTFAssetLoader';
import { Player } from '@/player/Player';
import { Sky } from '@/environment/Sky';
import { GameSettings } from '@/ui/HomeScreen';

export class MazeGame {
  private scene: THREE.Scene;
  private camera: THREE.PerspectiveCamera;
  private renderer: THREE.WebGLRenderer;
  private player: Player;
  private clock: THREE.Clock;
  private isRunning: boolean = false;
  private animationId: number | null = null;
  private settings: GameSettings;
  private gltfLoader: GLTFAssetLoader;
  private maze: THREE.Group | null = null;

  constructor(settings?: GameSettings) {
    this.settings = settings || {
      moveSpeed: 800, // Slightly slower for maze navigation
      mouseSensitivity: 0.002,
      graphicsQuality: 'medium'
    };

    this.gltfLoader = new GLTFAssetLoader();

    // Initialize Three.js components
    this.scene = new THREE.Scene();
    this.scene.fog = new THREE.Fog(0x222222, 10, 100); // Darker fog for maze atmosphere

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
    document.body.appendChild(this.renderer.domElement);

    this.clock = new THREE.Clock();

    // Initialize game components
    this.initLighting();
    this.loadMaze();

    // Initialize player with settings
    this.player = new Player(this.camera, this.scene, this.settings);

    // Initialize environment with darker sky for maze atmosphere
    const sky = new Sky(this.scene);

    // Handle window resize
    window.addEventListener('resize', () => this.onWindowResize());
  }

  private initLighting(): void {
    // Dimmer ambient light for maze atmosphere
    const ambientLight = new THREE.AmbientLight(0x404040, 0.3);
    this.scene.add(ambientLight);

    // Main directional light
    const directionalLight = new THREE.DirectionalLight(0xffffff, 0.8);
    directionalLight.position.set(0, 50, 0);
    directionalLight.castShadow = true;

    // Shadow settings for maze
    directionalLight.shadow.camera.left = -50;
    directionalLight.shadow.camera.right = 50;
    directionalLight.shadow.camera.top = 50;
    directionalLight.shadow.camera.bottom = -50;
    directionalLight.shadow.camera.near = 0.1;
    directionalLight.shadow.camera.far = 200;
    directionalLight.shadow.mapSize.width = 2048;
    directionalLight.shadow.mapSize.height = 2048;

    this.scene.add(directionalLight);

    // Add point lights at key locations for atmosphere
    const pointLight1 = new THREE.PointLight(0xffaa00, 0.5, 20);
    pointLight1.position.set(10, 5, 10);
    pointLight1.castShadow = true;
    this.scene.add(pointLight1);

    const pointLight2 = new THREE.PointLight(0xffaa00, 0.5, 20);
    pointLight2.position.set(-10, 5, -10);
    pointLight2.castShadow = true;
    this.scene.add(pointLight2);

    // Hemisphere light for subtle fill
    const hemisphereLight = new THREE.HemisphereLight(0x87CEEB, 0x2c2c2c, 0.2);
    this.scene.add(hemisphereLight);
  }

  private async loadMaze(): Promise<void> {
    try {
      console.log('Loading maze model...');
      this.maze = await this.gltfLoader.loadModel('/models/maze/scene.gltf');

      // Position and scale the maze appropriately
      this.maze.position.set(0, 0, 0);

      // Scale the maze to appropriate size
      const scale = 5; // Adjust this value based on the maze model size
      this.maze.scale.set(scale, scale, scale);

      // Ensure all maze parts cast and receive shadows
      this.maze.traverse((child) => {
        if (child instanceof THREE.Mesh) {
          child.castShadow = true;
          child.receiveShadow = true;

          // Ensure materials are visible and have proper properties
          if (child.material) {
            const material = child.material as THREE.Material;
            material.side = THREE.FrontSide;
          }
        }
      });

      this.scene.add(this.maze);
      console.log('Maze loaded successfully');

      // Position player at maze entrance or center
      this.positionPlayerAtStart();

    } catch (error) {
      console.error('Failed to load maze model:', error);

      // Create a simple procedural maze as fallback
      this.createFallbackMaze();
    }
  }

  private positionPlayerAtStart(): void {
    // Position player at a good starting point in the maze
    // You may need to adjust these coordinates based on your specific maze model
    this.camera.position.set(0, 2, 20); // Start at the edge of the maze
  }

  private createFallbackMaze(): void {
    // Create a simple procedural maze as fallback if GLTF loading fails
    const mazeGroup = new THREE.Group();

    const wallMaterial = new THREE.MeshLambertMaterial({ color: 0x666666 });
    const wallGeometry = new THREE.BoxGeometry(2, 4, 2);

    // Create a simple maze pattern
    const mazePattern = [
      [1,1,1,1,1,1,1,1,1,1],
      [1,0,0,0,1,0,0,0,0,1],
      [1,0,1,0,1,0,1,1,0,1],
      [1,0,1,0,0,0,1,0,0,1],
      [1,0,1,1,1,0,1,0,1,1],
      [1,0,0,0,0,0,0,0,0,1],
      [1,1,1,0,1,1,1,1,0,1],
      [1,0,0,0,1,0,0,0,0,1],
      [1,0,1,0,0,0,1,0,1,1],
      [1,1,1,1,1,1,1,1,1,1]
    ];

    for (let row = 0; row < mazePattern.length; row++) {
      for (let col = 0; col < mazePattern[row].length; col++) {
        if (mazePattern[row][col] === 1) {
          const wall = new THREE.Mesh(wallGeometry, wallMaterial);
          wall.position.set(col * 2 - 9, 2, row * 2 - 9);
          wall.castShadow = true;
          wall.receiveShadow = true;
          mazeGroup.add(wall);
        }
      }
    }

    // Add floor
    const floorGeometry = new THREE.PlaneGeometry(20, 20);
    const floorMaterial = new THREE.MeshLambertMaterial({ color: 0x333333 });
    const floor = new THREE.Mesh(floorGeometry, floorMaterial);
    floor.rotation.x = -Math.PI / 2;
    floor.receiveShadow = true;
    mazeGroup.add(floor);

    this.scene.add(mazeGroup);
    this.maze = mazeGroup;

    console.log('Fallback maze created');
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