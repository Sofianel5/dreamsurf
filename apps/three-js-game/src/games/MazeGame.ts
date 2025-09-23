import * as THREE from 'three';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';
import { Player } from '@/player/Player';
import { GameSettings } from '@/App';

export class MazeGame {
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
  private maze: THREE.Group | null = null;

  constructor(settings?: GameSettings) {
    this.settings = settings || {
      moveSpeed: 800,
      mouseSensitivity: 0.002,
      graphicsQuality: 'medium'
    };

    // Initialize Three.js components
    this.scene = new THREE.Scene();
    this.scene.background = new THREE.Color("#eeeeee");
    this.scene.fog = new THREE.FogExp2("#eeeeee", 0.02);

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

    // Initialize player first
    this.player = new Player(this.camera, this.scene, this.settings);

    // Create flat ground and set up terrain
    this.createFlatGround();

    // Set spawn position immediately after ground creation
    this.camera.position.set(0, 2, 20);

    // Load the GLTF maze
    this.loadMaze();

    // Handle window resize
    window.addEventListener('resize', () => this.onWindowResize());
  }

  private createFlatGround(): void {
    // Create a reasonably sized ground plane
    const groundGeometry = new THREE.PlaneGeometry(200, 200);
    const groundMaterial = new THREE.MeshLambertMaterial({
      color: 0x4a7c59, // Green ground
      side: THREE.DoubleSide // Fix single-sided collision issues
    });

    const ground = new THREE.Mesh(groundGeometry, groundMaterial);
    ground.rotation.x = -Math.PI / 2; // Rotate to be horizontal
    ground.position.y = -0.001; // Nudge down slightly to avoid exact y=0 issues
    ground.receiveShadow = true;
    ground.name = 'ground'; // Name it so we can find it later

    this.scene.add(ground);

    // Add lighting like GLTFGame
    const ambientLight = new THREE.AmbientLight(0xffffff, 0.5);
    this.scene.add(ambientLight);

    const directionalLight = new THREE.DirectionalLight(0xffffff, 2);
    directionalLight.position.set(100, 100, 100);
    directionalLight.castShadow = true;

    // Shadow settings like GLTFGame
    directionalLight.shadow.camera.far = 200;
    directionalLight.shadow.camera.left = -50;
    directionalLight.shadow.camera.right = 50;
    directionalLight.shadow.camera.top = 50;
    directionalLight.shadow.camera.bottom = -50;
    directionalLight.shadow.mapSize.set(2048, 2048);

    this.scene.add(directionalLight);

    // Set terrain for player collision
    this.player.setTerrainMesh(ground);

    console.log('Ground set for player collision');

    console.log('Flat ground created');
  }

  private loadMaze(): void {
    // Create maze material like GLTFGame does for terrain
    const mazeMaterial = new THREE.MeshPhongMaterial({
      color: "#5e875e"
    });

    // Load maze using GLTFLoader like GLTFGame does
    this.gltfLoader.load('/models/maze/scene.gltf', (gltf) => {
      console.log('GLTF maze loaded successfully');

      this.maze = gltf.scene;

      // Extract wall meshes for collision detection BEFORE scaling
      this.wallObjects = [];
      this.maze.traverse((child) => {
        if (child instanceof THREE.Mesh) {
          child.material = mazeMaterial;
          child.receiveShadow = true;
          child.castShadow = true;
          // Scale individual geometries like GLTFGame does
          child.geometry.scale(5, 5, 5);

          // Add mesh objects for collision
          this.wallObjects.push(child);
        }
      });

      // Position the maze on the ground (no group scaling)
      this.maze.position.set(0, 0, 0);

      this.scene.add(this.maze);

      // Set up wall collision for player
      this.player.setMazeWalls(this.wallObjects);

      // Camera position already set in constructor

      console.log('GLTF maze loaded with', this.wallObjects.length, 'collision objects');

    }, undefined, (error) => {
      console.error('Failed to load GLTF maze:', error);
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