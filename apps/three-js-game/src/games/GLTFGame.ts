import * as THREE from 'three';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';
import { MeshSurfaceSampler } from 'three/addons/math/MeshSurfaceSampler.js';
import { Player } from '@/player/Player';
import { GrassMaterial } from '@/GrassMaterial';
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

  // Grass-specific properties
  private loadingManager: THREE.LoadingManager;
  private textureLoader: THREE.TextureLoader;
  private gltfLoader: GLTFLoader;
  private grassMaterial: GrassMaterial;
  private grassGeometry = new THREE.BufferGeometry();
  private grassCount = 8000;
  private textures: { [key: string]: THREE.Texture } = {};

  constructor(settings?: GameSettings) {
    this.settings = settings || {
      moveSpeed: 1000,
      mouseSensitivity: 0.002,
      graphicsQuality: 'medium'
    };

    // Initialize loading manager
    this.loadingManager = new THREE.LoadingManager();
    this.textureLoader = new THREE.TextureLoader(this.loadingManager);
    this.gltfLoader = new GLTFLoader(this.loadingManager);

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

    // Initialize grass material
    this.grassMaterial = new GrassMaterial();

    // Initialize game components
    this.initLighting();
    this.setupTextures();
    this.loadModels();

    // Initialize player with settings
    this.player = new Player(this.camera, this.scene, this.settings);

    // Handle window resize
    window.addEventListener('resize', () => this.onWindowResize());
  }

  private initLighting(): void {
    // Ambient light
    const ambientLight = new THREE.AmbientLight(0xffffff, 0.5);
    this.scene.add(ambientLight);

    // Directional light (sun)
    const directionalLight = new THREE.DirectionalLight(0xffffff, 2);
    directionalLight.position.set(100, 100, 100);
    directionalLight.castShadow = true;

    // Shadow settings
    directionalLight.shadow.camera.far = 200;
    directionalLight.shadow.camera.left = -50;
    directionalLight.shadow.camera.right = 50;
    directionalLight.shadow.camera.top = 50;
    directionalLight.shadow.camera.bottom = -50;
    directionalLight.shadow.mapSize.set(2048, 2048);

    this.scene.add(directionalLight);
  }

  private setupTextures(): void {
    // Load noise texture for grass animation
    this.textures.perlinNoise = this.textureLoader.load("/models/perlinnoise.webp");
    this.textures.perlinNoise.wrapS = this.textures.perlinNoise.wrapT = THREE.RepeatWrapping;

    // Load grass alpha texture
    this.textures.grassAlpha = this.textureLoader.load("/models/grass.jpeg");

    // Setup grass material textures
    this.grassMaterial.setupTextures(
      this.textures.grassAlpha,
      this.textures.perlinNoise
    );
  }

  private addGrass(surfaceMesh: THREE.Mesh, grassGeometry: THREE.BufferGeometry): void {
    // Create a sampler for the terrain surface
    const sampler = new MeshSurfaceSampler(surfaceMesh)
      .setWeightAttribute("color")
      .build();

    // Create instanced mesh for grass
    const grassInstancedMesh = new THREE.InstancedMesh(
      grassGeometry,
      this.grassMaterial.material,
      this.grassCount
    );
    grassInstancedMesh.receiveShadow = true;

    const position = new THREE.Vector3();
    const quaternion = new THREE.Quaternion();
    const scale = new THREE.Vector3(1, 1, 1);

    const normal = new THREE.Vector3();
    const yAxis = new THREE.Vector3(0, 1, 0);
    const matrix = new THREE.Matrix4();

    // Sample randomly from the surface, creating grass instances
    for (let i = 0; i < this.grassCount; i++) {
      sampler.sample(position, normal);

      // Align the instance with the surface normal
      quaternion.setFromUnitVectors(yAxis, normal);

      // Create a random rotation around the y-axis
      const randomRotation = new THREE.Euler(0, Math.random() * Math.PI * 2, 0);
      const randomQuaternion = new THREE.Quaternion().setFromEuler(randomRotation);

      // Combine the alignment with the random rotation
      quaternion.multiply(randomQuaternion);

      // Set the matrix
      matrix.compose(position, quaternion, scale);
      grassInstancedMesh.setMatrixAt(i, matrix);
    }

    this.scene.add(grassInstancedMesh);
  }

  private loadModels(): void {
    // Create terrain material
    const terrainMat = new THREE.MeshPhongMaterial({
      color: "#5e875e"
    });

    // Load island terrain
    this.gltfLoader.load("/models/island.glb", (gltf) => {
      let terrainMesh: THREE.Mesh;

      gltf.scene.traverse((child) => {
        if (child instanceof THREE.Mesh) {
          child.material = terrainMat;
          child.receiveShadow = true;
          child.geometry.scale(3, 3, 3);
          terrainMesh = child;
        }
      });

      this.scene.add(gltf.scene);

      // Load grass model and add grass to terrain
      this.gltfLoader.load("/models/grassLODs.glb", (gltf) => {
        gltf.scene.traverse((child) => {
          if (child instanceof THREE.Mesh) {
            if (child.name.includes("LOD00")) {
              child.geometry.scale(5, 5, 5);
              this.grassGeometry = child.geometry;
            }
          }
        });

        // Add grass to the terrain
        if (terrainMesh) {
          this.addGrass(terrainMesh, this.grassGeometry);
        }
      });
    });

    // Load fluffy grass text
    const textMaterial = new THREE.MeshPhongMaterial({ color: 0x333333 });
    this.gltfLoader.load("/models/fluffy_grass_text.glb", (gltf) => {
      gltf.scene.traverse((child) => {
        if (child instanceof THREE.Mesh) {
          child.material = textMaterial;
          child.geometry.scale(3, 3, 3);
          child.position.y += 0.5;
          child.castShadow = true;
          child.receiveShadow = true;
        }
      });
      this.scene.add(gltf.scene);
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

    // Update grass material animation with elapsed time (not delta)
    this.grassMaterial.update(this.clock.getElapsedTime());

    // Update player
    this.player.update(deltaTime);

    // Render the scene
    this.renderer.render(this.scene, this.camera);
  }
}