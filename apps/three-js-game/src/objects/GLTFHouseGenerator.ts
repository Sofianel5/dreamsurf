import * as THREE from 'three';
import { GLTFAssetLoader } from '../loaders/GLTFAssetLoader';

export class GLTFHouseGenerator {
  private loader: GLTFAssetLoader;
  private containerModels = [
    '/models/containers/crate01_big.gltf',
    '/models/containers/crate01_small.gltf',
    '/models/containers/crate_square.gltf',
    '/models/containers/barrel01.gltf',
    '/models/containers/barrel_large_closed.gltf',
    '/models/containers/package_medium.gltf',
    '/models/containers/package_small.gltf'
  ];

  constructor() {
    this.loader = new GLTFAssetLoader();
  }

  async generateHouse(): Promise<THREE.Group> {
    const house = new THREE.Group();

    // Create a simple house base with procedural geometry
    const baseGeometry = new THREE.BoxGeometry(8, 6, 8);
    const baseMaterial = new THREE.MeshLambertMaterial({
      color: 0x8B7355,
      roughness: 0.9
    });
    const base = new THREE.Mesh(baseGeometry, baseMaterial);
    base.position.y = 3;
    base.castShadow = true;
    base.receiveShadow = true;
    house.add(base);

    // Create roof
    const roofGeometry = new THREE.ConeGeometry(6, 4, 4);
    const roofMaterial = new THREE.MeshLambertMaterial({
      color: 0x6B4423,
      roughness: 0.95
    });
    const roof = new THREE.Mesh(roofGeometry, roofMaterial);
    roof.position.y = 8;
    roof.rotation.y = Math.PI / 4;
    roof.castShadow = true;
    house.add(roof);

    // Add random containers around the house
    try {
      const numContainers = Math.floor(Math.random() * 3) + 1;
      for (let i = 0; i < numContainers; i++) {
        const randomModelPath = this.containerModels[Math.floor(Math.random() * this.containerModels.length)];
        const container = await this.loader.loadModel(randomModelPath);

        // Position container around the house
        const angle = (i / numContainers) * Math.PI * 2;
        const distance = 6 + Math.random() * 3;
        container.position.x = Math.cos(angle) * distance;
        container.position.z = Math.sin(angle) * distance;
        container.position.y = 0;

        // Random rotation
        container.rotation.y = Math.random() * Math.PI * 2;

        // Random scale
        const scale = 0.8 + Math.random() * 0.4;
        container.scale.set(scale, scale, scale);

        house.add(container);
      }
    } catch (error) {
      console.warn('Failed to load container models:', error);
    }

    return house;
  }

  async generateSimpleHouse(): Promise<THREE.Group> {
    // Fallback for synchronous generation
    const house = new THREE.Group();

    const baseGeometry = new THREE.BoxGeometry(8, 6, 8);
    const baseMaterial = new THREE.MeshLambertMaterial({
      color: 0x8B7355 + Math.random() * 0x111111,
      roughness: 0.9
    });
    const base = new THREE.Mesh(baseGeometry, baseMaterial);
    base.position.y = 3;
    base.castShadow = true;
    base.receiveShadow = true;
    house.add(base);

    const roofGeometry = new THREE.ConeGeometry(6, 4, 4);
    const roofMaterial = new THREE.MeshLambertMaterial({
      color: 0x6B4423,
      roughness: 0.95
    });
    const roof = new THREE.Mesh(roofGeometry, roofMaterial);
    roof.position.y = 8;
    roof.rotation.y = Math.PI / 4;
    roof.castShadow = true;
    house.add(roof);

    return house;
  }
}