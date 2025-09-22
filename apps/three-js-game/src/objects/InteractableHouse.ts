import * as THREE from 'three';
import { InteractableObject } from '@/objects/InteractableObject';

export class InteractableHouse extends InteractableObject {
  private isLightOn: boolean = false;
  private windowMaterial: THREE.MeshLambertMaterial;
  private originalWindowColor: number;

  constructor() {
    super();
    this.generateHouse();
  }

  private generateHouse(): void {
    // House dimensions
    const width = 10;
    const height = 6;
    const depth = 8;

    // Walls
    const wallGeometry = new THREE.BoxGeometry(width, height, depth);
    const wallMaterial = new THREE.MeshLambertMaterial({ color: 0x8b7355 });
    const walls = new THREE.Mesh(wallGeometry, wallMaterial);
    walls.position.y = height / 2;
    walls.castShadow = true;
    walls.receiveShadow = true;
    this.add(walls);

    // Roof
    const roofGeometry = new THREE.ConeGeometry(width * 0.8, height * 0.6, 4);
    const roofMaterial = new THREE.MeshLambertMaterial({ color: 0x8b4513 });
    const roof = new THREE.Mesh(roofGeometry, roofMaterial);
    roof.position.y = height + height * 0.3;
    roof.rotation.y = Math.PI / 4;
    roof.castShadow = true;
    this.add(roof);

    // Door
    const doorGeometry = new THREE.BoxGeometry(width * 0.2, height * 0.6, 0.2);
    const doorMaterial = new THREE.MeshLambertMaterial({ color: 0x4a3c28 });
    const door = new THREE.Mesh(doorGeometry, doorMaterial);
    door.position.set(0, height * 0.3, depth / 2 + 0.1);
    this.add(door);

    // Windows with interactive lighting
    const windowGeometry = new THREE.BoxGeometry(width * 0.15, height * 0.2, 0.2);
    this.windowMaterial = new THREE.MeshLambertMaterial({
      color: 0x87ceeb,
      emissive: 0x001122,
      emissiveIntensity: 0.1
    });
    this.originalWindowColor = 0x87ceeb;

    // Front windows
    const window1 = new THREE.Mesh(windowGeometry, this.windowMaterial);
    window1.position.set(-width * 0.25, height * 0.6, depth / 2 + 0.1);
    this.add(window1);

    const window2 = new THREE.Mesh(windowGeometry, this.windowMaterial);
    window2.position.set(width * 0.25, height * 0.6, depth / 2 + 0.1);
    this.add(window2);

    // Side windows
    const window3 = new THREE.Mesh(windowGeometry, this.windowMaterial);
    window3.position.set(width / 2 + 0.1, height * 0.6, 0);
    window3.rotation.y = Math.PI / 2;
    this.add(window3);

    // Add interaction prompt
    this.createInteractionPrompt();
  }

  private createInteractionPrompt(): void {
    const canvas = document.createElement('canvas');
    canvas.width = 512;
    canvas.height = 128;
    const context = canvas.getContext('2d')!;

    context.fillStyle = 'rgba(0, 0, 0, 0.8)';
    context.fillRect(0, 0, 512, 128);
    context.fillStyle = 'white';
    context.font = '32px Arial';
    context.textAlign = 'center';
    context.fillText('Press E to toggle lights', 256, 70);

    const texture = new THREE.CanvasTexture(canvas);
    const material = new THREE.SpriteMaterial({ map: texture, transparent: true });
    const sprite = new THREE.Sprite(material);
    sprite.position.set(0, 10, 0);
    sprite.scale.set(8, 2, 1);
    sprite.visible = false;
    sprite.name = 'prompt';
    this.add(sprite);
  }

  public showPrompt(): void {
    const prompt = this.getObjectByName('prompt');
    if (prompt) prompt.visible = true;
  }

  public hidePrompt(): void {
    const prompt = this.getObjectByName('prompt');
    if (prompt) prompt.visible = false;
  }

  public interact(): void {
    this.isLightOn = !this.isLightOn;

    if (this.isLightOn) {
      this.windowMaterial.emissive.setHex(0xffaa00);
      this.windowMaterial.emissiveIntensity = 0.8;

      // Add point light inside house
      const light = new THREE.PointLight(0xffaa00, 1, 20);
      light.position.set(0, 3, 0);
      light.name = 'houseLight';
      this.add(light);
    } else {
      this.windowMaterial.emissive.setHex(0x001122);
      this.windowMaterial.emissiveIntensity = 0.1;

      // Remove point light
      const light = this.getObjectByName('houseLight');
      if (light) this.remove(light);
    }
  }
}