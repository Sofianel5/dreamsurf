import * as THREE from 'three';
import { GLTFLoader } from 'three-stdlib';

export class GLTFAssetLoader {
  private loader: GLTFLoader;
  private loadedModels: Map<string, THREE.Group> = new Map();

  constructor() {
    this.loader = new GLTFLoader();
  }

  async loadModel(path: string): Promise<THREE.Group> {
    // Check cache first
    if (this.loadedModels.has(path)) {
      return this.loadedModels.get(path)!.clone();
    }

    return new Promise((resolve, reject) => {
      this.loader.load(
        path,
        (gltf) => {
          const model = gltf.scene;

          // Enable shadows for all meshes
          model.traverse((child) => {
            if (child instanceof THREE.Mesh) {
              child.castShadow = true;
              child.receiveShadow = true;
            }
          });

          // Cache the model
          this.loadedModels.set(path, model);

          // Return a clone to allow multiple instances
          resolve(model.clone());
        },
        (progress) => {
          console.log(`Loading ${path}: ${(progress.loaded / progress.total * 100).toFixed(0)}%`);
        },
        (error) => {
          console.error(`Error loading ${path}:`, error);
          reject(error);
        }
      );
    });
  }

  async loadMultipleModels(paths: string[]): Promise<THREE.Group[]> {
    return Promise.all(paths.map(path => this.loadModel(path)));
  }
}