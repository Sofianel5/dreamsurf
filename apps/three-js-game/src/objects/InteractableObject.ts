import * as THREE from 'three';

export abstract class InteractableObject extends THREE.Group {
  protected isHighlighted: boolean = false;
  protected originalMaterials: THREE.Material[] = [];
  protected highlightMaterial: THREE.MeshBasicMaterial;

  constructor() {
    super();
    this.highlightMaterial = new THREE.MeshBasicMaterial({
      color: 0xffff00,
      transparent: true,
      opacity: 0.3
    });
  }

  public highlight(): void {
    if (this.isHighlighted) return;

    this.isHighlighted = true;
    this.traverse((child) => {
      if (child instanceof THREE.Mesh) {
        this.originalMaterials.push(child.material);
        child.material = this.highlightMaterial;
      }
    });
  }

  public removeHighlight(): void {
    if (!this.isHighlighted) return;

    this.isHighlighted = false;
    let materialIndex = 0;
    this.traverse((child) => {
      if (child instanceof THREE.Mesh && materialIndex < this.originalMaterials.length) {
        child.material = this.originalMaterials[materialIndex];
        materialIndex++;
      }
    });
    this.originalMaterials = [];
  }

  public abstract interact(): void;
}