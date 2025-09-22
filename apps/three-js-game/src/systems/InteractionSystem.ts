import * as THREE from 'three';
import { InteractableObject } from '@/objects/InteractableObject';

export class InteractionSystem {
  private camera: THREE.Camera;
  private scene: THREE.Scene;
  private raycaster: THREE.Raycaster;
  private currentTarget: InteractableObject | null = null;
  private interactionRange: number = 10;

  constructor(camera: THREE.Camera, scene: THREE.Scene) {
    this.camera = camera;
    this.scene = scene;
    this.raycaster = new THREE.Raycaster();

    this.initControls();
  }

  private initControls(): void {
    document.addEventListener('keydown', (event) => {
      if (event.code === 'KeyE' && this.currentTarget) {
        this.currentTarget.interact();
      }
    });
  }

  public update(): void {
    // Cast ray from camera forward
    this.raycaster.setFromCamera(new THREE.Vector2(0, 0), this.camera);

    // Find interactable objects
    const interactables: InteractableObject[] = [];
    this.scene.traverse((object) => {
      if (object instanceof InteractableObject) {
        interactables.push(object);
      }
    });

    // Check for intersections
    const intersects = this.raycaster.intersectObjects(interactables, true);

    let newTarget: InteractableObject | null = null;

    if (intersects.length > 0) {
      // Find the closest interactable object
      for (const intersect of intersects) {
        if (intersect.distance <= this.interactionRange) {
          let parent = intersect.object.parent;
          while (parent) {
            if (parent instanceof InteractableObject) {
              newTarget = parent;
              break;
            }
            parent = parent.parent;
          }
          if (newTarget) break;
        }
      }
    }

    // Update target highlighting
    if (this.currentTarget !== newTarget) {
      if (this.currentTarget) {
        this.currentTarget.removeHighlight();
        if ('hidePrompt' in this.currentTarget) {
          (this.currentTarget as any).hidePrompt();
        }
      }

      this.currentTarget = newTarget;

      if (this.currentTarget) {
        this.currentTarget.highlight();
        if ('showPrompt' in this.currentTarget) {
          (this.currentTarget as any).showPrompt();
        }
      }
    }
  }
}