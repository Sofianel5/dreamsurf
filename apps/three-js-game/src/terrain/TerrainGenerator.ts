import * as THREE from 'three';

export class TerrainGenerator {
  private size: number = 200;
  private segments: number = 100;

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

    // Create textured material
    const texture = this.createGroundTexture();
    const material = new THREE.MeshLambertMaterial({
      map: texture,
      side: THREE.DoubleSide,
    });

    const mesh = new THREE.Mesh(geometry, material);
    mesh.receiveShadow = true;

    return mesh;
  }

  private createGroundTexture(): THREE.Texture {
    const canvas = document.createElement('canvas');
    canvas.width = 512;
    canvas.height = 512;
    const context = canvas.getContext('2d')!;

    // Create a grass-like texture
    const gradient = context.createLinearGradient(0, 0, 0, 512);
    gradient.addColorStop(0, '#3a5f3a');
    gradient.addColorStop(0.5, '#4a6f4a');
    gradient.addColorStop(1, '#2a4f2a');

    context.fillStyle = gradient;
    context.fillRect(0, 0, 512, 512);

    // Add some noise for texture
    for (let i = 0; i < 5000; i++) {
      const x = Math.random() * 512;
      const y = Math.random() * 512;
      const brightness = 0.3 + Math.random() * 0.2;
      context.fillStyle = `rgba(${brightness * 100}, ${brightness * 150}, ${brightness * 100}, 0.5)`;
      context.fillRect(x, y, 2, 2);
    }

    const texture = new THREE.CanvasTexture(canvas);
    texture.wrapS = THREE.RepeatWrapping;
    texture.wrapT = THREE.RepeatWrapping;
    texture.repeat.set(20, 20);

    return texture;
  }
}