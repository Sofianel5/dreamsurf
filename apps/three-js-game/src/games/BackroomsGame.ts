import * as THREE from 'three';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';
import { Player } from '@/player/Player';
import { GameSettings } from '@/App';

export class BackroomsGame {
  private scene: THREE.Scene;
  private camera: THREE.PerspectiveCamera;
  private renderer: THREE.WebGLRenderer;
  private player: Player;
  private clock: THREE.Clock;
  private isRunning: boolean = false;
  private animationId: number | null = null;
  private settings: GameSettings;
  private wallObjects: THREE.Mesh[] = [];
  private gltfLoader: GLTFLoader;
  private backrooms: THREE.Group | null = null;

  constructor(settings?: GameSettings) {
    this.settings = settings || {
      moveSpeed: 1000,
      mouseSensitivity: 0.002,
      graphicsQuality: 'medium'
    };

    // Initialize Three.js components
    this.scene = new THREE.Scene();
    this.scene.background = new THREE.Color("#f4f1a8"); // Backrooms yellow
    this.scene.fog = new THREE.FogExp2("#f4f1a8", 0.02); // Moderate fog for atmosphere

    this.camera = new THREE.PerspectiveCamera(
      75,
      window.innerWidth / window.innerHeight,
      0.1,
      1000
    );

    this.renderer = new THREE.WebGLRenderer({
      antialias: true,
      alpha: true,
      precision: "highp"
    });
    this.renderer.setSize(window.innerWidth, window.innerHeight);
    this.renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    this.renderer.shadowMap.enabled = true;
    this.renderer.shadowMap.autoUpdate = true;
    this.renderer.shadowMap.type = THREE.PCFSoftShadowMap;
    this.renderer.outputColorSpace = THREE.SRGBColorSpace;
    this.renderer.toneMapping = THREE.ACESFilmicToneMapping;
    this.scene.frustumCulled = true;

    this.clock = new THREE.Clock();

    // Initialize GLTF loader
    this.gltfLoader = new GLTFLoader();

    // Initialize lighting for backrooms atmosphere
    this.initLighting();

    // Initialize player
    this.player = new Player(this.camera, this.scene, this.settings);

    // Create ground and load backrooms
    this.createFloor();
    this.loadBackrooms();

    // Handle window resize
    window.addEventListener('resize', () => this.onWindowResize());
  }

  private initLighting(): void {
    // Dim ambient light for eerie atmosphere
    const ambientLight = new THREE.AmbientLight(0xffffff, 0.3);
    this.scene.add(ambientLight);

    // Fluorescent-style lighting
    const light1 = new THREE.DirectionalLight(0xfff2cc, 0.8);
    light1.position.set(10, 20, 10);
    light1.castShadow = true;
    light1.shadow.camera.far = 100;
    light1.shadow.camera.left = -50;
    light1.shadow.camera.right = 50;
    light1.shadow.camera.top = 50;
    light1.shadow.camera.bottom = -50;
    light1.shadow.mapSize.set(2048, 2048);
    this.scene.add(light1);

    // Additional fluorescent lights for backrooms feel
    const light2 = new THREE.DirectionalLight(0xfff2cc, 0.6);
    light2.position.set(-10, 20, -10);
    light2.castShadow = true;
    light2.shadow.camera.far = 100;
    light2.shadow.camera.left = -50;
    light2.shadow.camera.right = 50;
    light2.shadow.camera.top = 50;
    light2.shadow.camera.bottom = -50;
    light2.shadow.mapSize.set(1024, 1024);
    this.scene.add(light2);
  }

  private createFloor(): void {
    // Create a minimal floor plane for collision
    const floorGeometry = new THREE.PlaneGeometry(1000, 1000);
    const floorMaterial = new THREE.MeshLambertMaterial({
      color: 0xd4d4aa,
      side: THREE.DoubleSide,
      transparent: true,
      opacity: 0.1 // Make it nearly invisible since the model should have its own floor
    });

    const floor = new THREE.Mesh(floorGeometry, floorMaterial);
    floor.rotation.x = -Math.PI / 2;
    floor.position.y = 0;
    floor.receiveShadow = true;
    floor.name = 'floor';

    this.scene.add(floor);

    // Set terrain for player collision
    this.player.setTerrainMesh(floor);

    console.log('Backrooms floor created');
  }

  private loadBackrooms(): void {
    // Load backrooms level 0 model
    this.gltfLoader.load('/models/backrooms_level_0/scene.gltf', (gltf) => {
      console.log('Backrooms GLTF loaded successfully');

      this.backrooms = gltf.scene;

      // Extract wall/ceiling meshes for collision detection
      this.wallObjects = [];
      this.backrooms.traverse((child) => {
        if (child instanceof THREE.Mesh) {
          // Keep original materials for backrooms atmosphere
          child.receiveShadow = true;
          child.castShadow = true;

          // Scale geometry appropriately for backrooms level
          child.geometry.scale(10, 10, 10);

          // Add mesh objects for collision (walls, ceiling, etc.)
          this.wallObjects.push(child);
        }
      });

      // Position the backrooms
      this.backrooms.position.set(0, 0, 0);
      this.scene.add(this.backrooms);

      // Set up wall collision for player
      this.player.setMazeWalls(this.wallObjects);

      // Set spawn position at player height inside the level
      this.camera.position.set(0, 1.7, 0);

      console.log('Backrooms loaded with', this.wallObjects.length, 'collision objects');

    }, undefined, (error) => {
      console.error('Failed to load Backrooms GLTF:', error);
    });
  }

  private onWindowResize(): void {
    this.camera.aspect = window.innerWidth / window.innerHeight;
    this.camera.updateProjectionMatrix();
    this.renderer.setSize(window.innerWidth, window.innerHeight);
  }

  public start(): void {
    if (!this.isRunning) {
      this.isRunning = true;

      // Ensure the renderer canvas can receive focus for keyboard events
      this.renderer.domElement.setAttribute('tabindex', '0');
      this.renderer.domElement.focus();

      // Add click listener for pointer lock
      this.renderer.domElement.addEventListener('click', () => {
        document.body.requestPointerLock();
      });

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