import * as THREE from 'three';
import { AssetLoader } from '../loaders/AssetLoader';

export class VegetationGenerator {
  private loader: AssetLoader;

  constructor() {
    this.loader = AssetLoader.getInstance();
  }

  async generateIvy(position: THREE.Vector3): Promise<THREE.Group | null> {
    try {
      const ivy = await this.loader.loadModel('/models/nature/ivy_part08.gltf');
      ivy.position.copy(position);

      // Random rotation
      ivy.rotation.y = Math.random() * Math.PI * 2;

      // Random scale
      const scale = 0.5 + Math.random() * 1.5;
      ivy.scale.set(scale, scale, scale);

      return ivy;
    } catch (error) {
      console.warn('Failed to load ivy model:', error);
      return null;
    }
  }

  async generateFox(position: THREE.Vector3): Promise<THREE.Group | null> {
    try {
      const fox = await this.loader.loadModel('/models/fox/Fox.gltf');
      fox.position.copy(position);

      // Random rotation
      fox.rotation.y = Math.random() * Math.PI * 2;

      // Scale down the fox
      const scale = 0.02;
      fox.scale.set(scale, scale, scale);

      return fox;
    } catch (error) {
      console.warn('Failed to load fox model:', error);
      return null;
    }
  }

  async generateRealisticGrass(position: THREE.Vector3): Promise<THREE.Group | null> {
    try {
      const grass = await this.loader.loadModel('/models/realistic_grass/scene.gltf');
      grass.position.copy(position);

      // Random rotation around Y axis
      grass.rotation.y = Math.random() * Math.PI * 2;

      // Much larger scale for the grass model - the model seems very small
      const scale = 1.0 + Math.random() * 0.5; // Much bigger scale for visibility
      grass.scale.set(scale, scale, scale);

      // Ensure all meshes in the grass model have proper materials and are visible
      grass.traverse((child) => {
        if (child instanceof THREE.Mesh) {
          // Make sure shadows work
          child.castShadow = true;
          child.receiveShadow = true;

          // Ensure the mesh is visible
          child.visible = true;

          // If material exists, make sure it's properly configured
          if (child.material) {
            const mat = child.material as THREE.MeshStandardMaterial;
            // Ensure material is visible
            mat.visible = true;
            mat.side = THREE.DoubleSide; // Show both sides
          }
        }
      });

      return grass;
    } catch (error) {
      console.warn('Failed to generate realistic grass:', error);
      return this.generateProceduralGrass(position);
    }
  }

  generateProceduralGrass(position: THREE.Vector3): THREE.Group {
    const grass = new THREE.Group();

    // Create multiple grass blades
    const bladeCount = 5 + Math.floor(Math.random() * 10);
    const bladeGeometry = new THREE.ConeGeometry(0.1, 2, 3);
    const bladeMaterial = new THREE.MeshLambertMaterial({
      color: 0x2d5016,
      side: THREE.DoubleSide
    });

    for (let i = 0; i < bladeCount; i++) {
      const blade = new THREE.Mesh(bladeGeometry, bladeMaterial);
      blade.position.x = (Math.random() - 0.5) * 2;
      blade.position.z = (Math.random() - 0.5) * 2;
      blade.position.y = 1;
      blade.rotation.x = (Math.random() - 0.5) * 0.3;
      blade.rotation.z = (Math.random() - 0.5) * 0.3;
      const scale = 0.5 + Math.random() * 0.5;
      blade.scale.set(scale, scale, scale);
      grass.add(blade);
    }

    grass.position.copy(position);
    return grass;
  }
}