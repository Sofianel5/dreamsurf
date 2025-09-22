import * as THREE from 'three';
import { AssetLoader } from '@/loaders/AssetLoader';
import { Player } from '@/player/Player';
import { Sky } from '@/environment/Sky';
import { GameSettings } from '@/ui/HomeScreen';

export class GLTFGame {
  private scene: THREE.Scene;
  private camera: THREE.PerspectiveCamera;
  private renderer: THREE.WebGLRenderer;
  private player: Player;
  private clock: THREE.Clock;
  private isRunning: boolean = false;
  private animationId: number | null = null;
  private settings: GameSettings;
  private gltfLoader: GLTFAssetLoader;

  constructor(settings?: GameSettings) {
    this.settings = settings || {
      moveSpeed: 1000,
      mouseSensitivity: 0.002,
      graphicsQuality: 'medium'
    };

    this.gltfLoader = new GLTFAssetLoader();

    // Initialize Three.js components
    this.scene = new THREE.Scene();
    this.scene.fog = new THREE.Fog(0x87CEEB, 50, 500);

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
    this.initFlatTerrain();

    // Initialize player with settings for flat terrain
    this.player = new Player(this.camera, this.scene, this.settings);

    // Set player position just above ground
    this.camera.position.set(0, 1, 0);

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
    directionalLight.shadow.camera.left = -150;
    directionalLight.shadow.camera.right = 150;
    directionalLight.shadow.camera.top = 150;
    directionalLight.shadow.camera.bottom = -150;
    directionalLight.shadow.camera.near = 0.1;
    directionalLight.shadow.camera.far = 300;
    directionalLight.shadow.mapSize.width = 2048;
    directionalLight.shadow.mapSize.height = 2048;

    this.scene.add(directionalLight);

    // Add some hemisphere light for better ambient
    const hemisphereLight = new THREE.HemisphereLight(0x87CEEB, 0x545454, 0.4);
    this.scene.add(hemisphereLight);
  }

  private async initFlatTerrain(): Promise<void> {
    // Create terrain entirely made of grass GLTF models
    await this.createGrassTerrain();
  }


  private async createGrassTerrain(): Promise<void> {
    // 1) Load ONE grass model
    const gltf = await this.gltfLoader.loadModel('/models/grass.glb'); // convert to .glb if you can
    // Find a single mesh (ideally 1 material). If your asset has multiple child meshes,
    // pre-bake to a single mesh+material first (see notes below).
    let src: THREE.Mesh | null = null;
    gltf.traverse((o: any) => { if (o.isMesh && !src) src = o; });
    if (!src) throw new Error('Grass mesh not found');
    const geom = (src as THREE.Mesh).geometry;
    const mat = (src as THREE.Mesh).material;

    // 2) Create instances
    const terrainSize = 200;
    const spacing = 4;
    const nx = Math.floor(terrainSize / spacing);
    const nz = Math.floor(terrainSize / spacing);
    const COUNT = nx * nz;

    const grass = new THREE.InstancedMesh(geom, mat, COUNT);
    // If static, omit DynamicDrawUsage for max perf
    // grass.instanceMatrix.setUsage(THREE.DynamicDrawUsage);

    const m = new THREE.Matrix4();
    let i = 0;
    for (let x = 0; x < nx; x++) {
      for (let z = 0; z < nz; z++) {
        const px = (x - nx / 2) * spacing + (Math.random() - 0.5) * 2;
        const pz = (z - nz / 2) * spacing + (Math.random() - 0.5) * 2;
        const rot = Math.random() * Math.PI * 2;
        const s = 0.8 + Math.random() * 0.4;
        m.identity()
          .makeRotationY(rot)
          .setPosition(px, 0, pz);
        m.multiply(new THREE.Matrix4().makeScale(s, s, s));
        grass.setMatrixAt(i++, m);
      }
    }
    grass.instanceMatrix.needsUpdate = true;
    grass.castShadow = true;
    grass.receiveShadow = true;
    this.scene.add(grass);
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