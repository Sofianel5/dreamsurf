import * as THREE from 'three';
import { InteractableObject } from '@/objects/InteractableObject';

export class Collectible extends InteractableObject {
  private rotationSpeed: number = 2;
  private bobSpeed: number = 3;
  private bobAmount: number = 0.5;
  private time: number = 0;
  private initialY: number = 0;

  constructor(type: 'gem' | 'coin' = 'gem') {
    super();
    this.generateCollectible(type);
  }

  private generateCollectible(type: string): void {
    let geometry: THREE.BufferGeometry;
    let material: THREE.Material;

    if (type === 'gem') {
      geometry = new THREE.OctahedronGeometry(1);
      material = new THREE.MeshLambertMaterial({
        color: 0x00ff88,
        emissive: 0x004422,
        emissiveIntensity: 0.2
      });
    } else {
      geometry = new THREE.CylinderGeometry(1, 1, 0.2, 12);
      material = new THREE.MeshLambertMaterial({
        color: 0xffdd00,
        emissive: 0x332200,
        emissiveIntensity: 0.1
      });
    }

    const mesh = new THREE.Mesh(geometry, material);
    mesh.castShadow = true;
    mesh.position.y = 2;
    this.add(mesh);

    this.initialY = this.position.y;

    // Add particle effect
    this.addParticleEffect();
  }

  private addParticleEffect(): void {
    const particleCount = 10;
    const geometry = new THREE.BufferGeometry();
    const positions = new Float32Array(particleCount * 3);

    for (let i = 0; i < particleCount * 3; i += 3) {
      positions[i] = (Math.random() - 0.5) * 4;
      positions[i + 1] = Math.random() * 3;
      positions[i + 2] = (Math.random() - 0.5) * 4;
    }

    geometry.setAttribute('position', new THREE.BufferAttribute(positions, 3));

    const material = new THREE.PointsMaterial({
      color: 0xffffff,
      size: 0.1,
      transparent: true,
      opacity: 0.6
    });

    const particles = new THREE.Points(geometry, material);
    this.add(particles);
  }

  public update(deltaTime: number): void {
    this.time += deltaTime;

    // Rotate
    this.rotation.y += this.rotationSpeed * deltaTime;

    // Bob up and down
    this.position.y = this.initialY + Math.sin(this.time * this.bobSpeed) * this.bobAmount;
  }

  public interact(): void {
    // Create collection effect
    this.createCollectionEffect();

    // Remove from scene after a short delay
    setTimeout(() => {
      this.removeFromParent();
    }, 200);

    // Dispatch collection event
    window.dispatchEvent(new CustomEvent('collectibleCollected', {
      detail: { type: 'gem', points: 10 }
    }));
  }

  private createCollectionEffect(): void {
    const geometry = new THREE.SphereGeometry(2, 8, 6);
    const material = new THREE.MeshBasicMaterial({
      color: 0xffff00,
      transparent: true,
      opacity: 0.8
    });

    const effect = new THREE.Mesh(geometry, material);
    effect.position.copy(this.position);
    this.parent?.add(effect);

    // Animate the effect
    const startTime = Date.now();
    const animate = () => {
      const elapsed = Date.now() - startTime;
      const progress = elapsed / 300;

      if (progress < 1) {
        effect.scale.setScalar(1 + progress * 2);
        material.opacity = 0.8 * (1 - progress);
        requestAnimationFrame(animate);
      } else {
        effect.removeFromParent();
      }
    };
    animate();
  }
}