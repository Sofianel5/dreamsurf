import * as THREE from 'three';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';

export class AssetLoader {
  private static instance: AssetLoader;
  private loader: GLTFLoader;
  private textureLoader: THREE.TextureLoader;
  private cache: Map<string, any> = new Map();

  private constructor() {
    this.loader = new GLTFLoader();
    this.textureLoader = new THREE.TextureLoader();
  }

  public static getInstance(): AssetLoader {
    if (!AssetLoader.instance) {
      AssetLoader.instance = new AssetLoader();
    }
    return AssetLoader.instance;
  }

  public async loadModel(path: string): Promise<THREE.Group> {
    if (this.cache.has(path)) {
      return this.cache.get(path).clone();
    }

    return new Promise((resolve, reject) => {
      this.loader.load(
        path,
        (gltf) => {
          // Setup shadows for all meshes
          gltf.scene.traverse((child) => {
            if (child instanceof THREE.Mesh) {
              child.castShadow = true;
              child.receiveShadow = true;
            }
          });

          this.cache.set(path, gltf.scene);
          resolve(gltf.scene.clone());
        },
        undefined,
        reject
      );
    });
  }

  public loadTexture(path: string): THREE.Texture {
    if (this.cache.has(path)) {
      return this.cache.get(path);
    }

    const texture = this.textureLoader.load(path);
    this.cache.set(path, texture);
    return texture;
  }

  public createMaterial(options: {
    color?: number;
    map?: string;
    normalMap?: string;
    roughnessMap?: string;
    metalness?: number;
    roughness?: number;
  }): THREE.MeshStandardMaterial {
    const material = new THREE.MeshStandardMaterial({
      color: options.color || 0xffffff,
      metalness: options.metalness || 0,
      roughness: options.roughness || 0.5,
    });

    if (options.map) {
      material.map = this.loadTexture(options.map);
    }

    if (options.normalMap) {
      material.normalMap = this.loadTexture(options.normalMap);
    }

    if (options.roughnessMap) {
      material.roughnessMap = this.loadTexture(options.roughnessMap);
    }

    return material;
  }
}