import * as THREE from 'three';
import { TerrainGenerator } from '@/terrain/TerrainGenerator';
import { HouseGenerator } from '@/objects/HouseGenerator';
import { Player } from '@/player/Player';
import { Sky } from '@/environment/Sky';

export class Game {
  private scene: THREE.Scene;
  private camera: THREE.PerspectiveCamera;
  private renderer: THREE.WebGLRenderer;
  private player: Player;
  private clock: THREE.Clock;

  constructor() {
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
    this.initTerrain();
    this.initObjects();

    // Initialize player
    this.player = new Player(this.camera, this.scene);

    // Initialize environment
    const sky = new Sky(this.scene);

    // Handle window resize
    window.addEventListener('resize', () => this.onWindowResize());
  }

  private initLighting(): void {
    // Ambient light
    const ambientLight = new THREE.AmbientLight(0x404040, 0.5);
    this.scene.add(ambientLight);

    // Directional light (sun)
    const directionalLight = new THREE.DirectionalLight(0xffffff, 1);
    directionalLight.position.set(50, 100, 50);
    directionalLight.castShadow = true;

    // Shadow settings
    directionalLight.shadow.camera.left = -100;
    directionalLight.shadow.camera.right = 100;
    directionalLight.shadow.camera.top = 100;
    directionalLight.shadow.camera.bottom = -100;
    directionalLight.shadow.camera.near = 0.1;
    directionalLight.shadow.camera.far = 300;
    directionalLight.shadow.mapSize.width = 2048;
    directionalLight.shadow.mapSize.height = 2048;

    this.scene.add(directionalLight);

    // Add some hemisphere light for better ambient
    const hemisphereLight = new THREE.HemisphereLight(0x87CEEB, 0x545454, 0.5);
    this.scene.add(hemisphereLight);
  }

  private initTerrain(): void {
    const terrainGenerator = new TerrainGenerator();
    const terrain = terrainGenerator.generate();
    this.scene.add(terrain);
  }

  private initObjects(): void {
    const houseGenerator = new HouseGenerator();

    // Define house positions
    const housePositions = [
      { x: 0, z: 0 },
      { x: 40, z: 30 },
      { x: -45, z: -20 },
      { x: 25, z: -40 },
      { x: -30, z: 35 },
      { x: 60, z: -10 },
      { x: -60, z: 5 },
    ];

    // Generate houses at each position
    housePositions.forEach(pos => {
      const y = this.getTerrainHeightAt(pos.x, pos.z);
      const house = houseGenerator.generateHouse();
      house.position.set(pos.x, y, pos.z);
      house.rotation.y = Math.random() * Math.PI * 2;
      this.scene.add(house);
    });

    // Add some trees
    this.addTrees();
  }

  private addTrees(): void {
    const treeGeometry = new THREE.ConeGeometry(2, 8, 6);
    const treeMaterial = new THREE.MeshLambertMaterial({ color: 0x0d5d0d });
    const trunkGeometry = new THREE.CylinderGeometry(0.5, 0.5, 3);
    const trunkMaterial = new THREE.MeshLambertMaterial({ color: 0x8b4513 });

    for (let i = 0; i < 50; i++) {
      const x = (Math.random() - 0.5) * 200;
      const z = (Math.random() - 0.5) * 200;
      const y = this.getTerrainHeightAt(x, z);

      // Skip if too close to center (where houses are)
      if (Math.abs(x) < 20 && Math.abs(z) < 20) continue;

      const tree = new THREE.Group();

      // Trunk
      const trunk = new THREE.Mesh(trunkGeometry, trunkMaterial);
      trunk.position.y = 1.5;
      trunk.castShadow = true;
      tree.add(trunk);

      // Leaves
      const leaves = new THREE.Mesh(treeGeometry, treeMaterial);
      leaves.position.y = 5;
      leaves.castShadow = true;
      tree.add(leaves);

      tree.position.set(x, y, z);
      this.scene.add(tree);
    }
  }

  private getTerrainHeightAt(x: number, z: number): number {
    // Calculate terrain height using the same formula as TerrainGenerator
    const terrainSize = 200;
    const fx = (x / terrainSize + 0.5);
    const fz = (z / terrainSize + 0.5);

    const height =
      Math.sin(fx * 8) * Math.sin(fz * 8) * 3 +
      Math.sin(fx * 16) * Math.sin(fz * 16) * 1.5 +
      Math.sin(fx * 4) * Math.sin(fz * 4) * 5;

    return height;
  }

  private onWindowResize(): void {
    this.camera.aspect = window.innerWidth / window.innerHeight;
    this.camera.updateProjectionMatrix();
    this.renderer.setSize(window.innerWidth, window.innerHeight);
  }

  public start(): void {
    this.animate();
  }

  private animate = (): void => {
    requestAnimationFrame(this.animate);

    const deltaTime = this.clock.getDelta();

    // Update player
    this.player.update(deltaTime);

    // Render the scene
    this.renderer.render(this.scene, this.camera);
  }
}