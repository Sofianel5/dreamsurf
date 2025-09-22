import * as THREE from 'three';

export class TerrainGenerator {
  private size: number = 200;
  private segments: number = 200; // Increased for better texture mapping on slopes
  private textureLoader: THREE.TextureLoader;

  constructor() {
    this.textureLoader = new THREE.TextureLoader();
  }

  public generate(): THREE.Mesh {
    // Create plane geometry
    const geometry = new THREE.PlaneGeometry(
      this.size,
      this.size,
      this.segments,
      this.segments
    );

    // Rotate to be horizontal
    geometry.rotateX(-Math.PI / 2);

    // Get vertex positions
    const positions = geometry.attributes.position;

    // Generate hills using sine waves
    for (let i = 0; i < positions.count; i++) {
      const x = positions.getX(i);
      const z = positions.getZ(i);

      // Normalize coordinates
      const fx = (x / this.size + 0.5);
      const fz = (z / this.size + 0.5);

      // Calculate height using combined sine waves
      const height =
        Math.sin(fx * 8) * Math.sin(fz * 8) * 3 +      // Main hills
        Math.sin(fx * 16) * Math.sin(fz * 16) * 1.5 +  // Smaller variations
        Math.sin(fx * 4) * Math.sin(fz * 4) * 5;       // Large rolling hills

      positions.setY(i, height);
    }

    // Update normals for proper lighting
    geometry.computeVertexNormals();

    // Load the grass texture directly
    const grassTexture = this.textureLoader.load('/models/realistic_grass/textures/grasss_baseColor.jpeg');
    grassTexture.wrapS = THREE.RepeatWrapping;
    grassTexture.wrapT = THREE.RepeatWrapping;
    grassTexture.repeat.set(50, 50); // Tile the texture across the terrain
    grassTexture.anisotropy = 16; // Improve texture quality at angles
    grassTexture.colorSpace = THREE.SRGBColorSpace;

    // Create material with the grass texture
    const material = new THREE.MeshStandardMaterial({
      map: grassTexture,
      roughness: 0.9,
      metalness: 0,
      side: THREE.DoubleSide,
    });

    const mesh = new THREE.Mesh(geometry, material);
    mesh.receiveShadow = true;

    return mesh;
  }
}