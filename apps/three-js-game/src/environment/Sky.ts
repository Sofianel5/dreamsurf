import * as THREE from 'three';

export class Sky {
  constructor(scene: THREE.Scene) {
    this.createSky(scene);
  }

  private createSky(scene: THREE.Scene): void {
    // Create sky sphere
    const skyGeometry = new THREE.SphereGeometry(300, 32, 16);

    // Create gradient texture for sky
    const canvas = document.createElement('canvas');
    canvas.width = 256;
    canvas.height = 256;
    const context = canvas.getContext('2d')!;

    // Create sky gradient
    const gradient = context.createLinearGradient(0, 0, 0, 256);
    gradient.addColorStop(0, '#87CEEB'); // Sky blue at top
    gradient.addColorStop(0.7, '#98D8E8'); // Lighter blue
    gradient.addColorStop(1, '#F0F8FF'); // Very light blue at horizon

    context.fillStyle = gradient;
    context.fillRect(0, 0, 256, 256);

    const skyTexture = new THREE.CanvasTexture(canvas);

    const skyMaterial = new THREE.MeshBasicMaterial({
      map: skyTexture,
      side: THREE.BackSide,
      fog: false
    });

    const sky = new THREE.Mesh(skyGeometry, skyMaterial);
    scene.add(sky);

    // Add some clouds
    this.addClouds(scene);
  }

  private addClouds(scene: THREE.Scene): void {
    const cloudGeometry = new THREE.SphereGeometry(5, 8, 6);
    const cloudMaterial = new THREE.MeshBasicMaterial({
      color: 0xffffff,
      transparent: true,
      opacity: 0.7,
      fog: false
    });

    for (let i = 0; i < 20; i++) {
      const cloud = new THREE.Group();

      // Create cloud from multiple spheres
      for (let j = 0; j < 5; j++) {
        const cloudPart = new THREE.Mesh(cloudGeometry, cloudMaterial);
        cloudPart.position.set(
          (Math.random() - 0.5) * 15,
          Math.random() * 5,
          (Math.random() - 0.5) * 15
        );
        cloudPart.scale.setScalar(0.5 + Math.random() * 0.5);
        cloud.add(cloudPart);
      }

      // Position clouds around the sky
      const angle = (i / 20) * Math.PI * 2;
      const radius = 150 + Math.random() * 50;
      cloud.position.set(
        Math.cos(angle) * radius,
        50 + Math.random() * 30,
        Math.sin(angle) * radius
      );

      scene.add(cloud);
    }
  }
}