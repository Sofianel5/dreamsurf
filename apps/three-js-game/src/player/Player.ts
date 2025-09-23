import * as THREE from 'three';
import { GameSettings } from '@/App';

export class Player {
  private camera: THREE.PerspectiveCamera;
  private scene: THREE.Scene;
  private velocity: THREE.Vector3 = new THREE.Vector3();
  private direction: THREE.Vector3 = new THREE.Vector3();
  private keys: { [key: string]: boolean } = {};
  private mouse: { x: number; y: number } = { x: 0, y: 0 };
  private isMouseLocked: boolean = false;

  private moveSpeed: number = 1000;
  private mouseSensitivity: number = 0.002;
  private jumpSpeed: number = 10;
  private gravity: number = -35;
  private isGrounded: boolean = true;

  constructor(camera: THREE.PerspectiveCamera, scene: THREE.Scene, settings?: GameSettings) {
    this.camera = camera;
    this.scene = scene;

    // Apply settings if provided
    if (settings) {
      this.updateSettings(settings);
    }

    // Set initial position for flat terrain - may be overridden by game-specific positioning
    this.camera.position.set(0, 2, 0);

    this.initControls();
  }

  public updateSettings(settings: GameSettings): void {
    this.moveSpeed = settings.moveSpeed;
    this.mouseSensitivity = settings.mouseSensitivity;
  }

  private initControls(): void {
    // Keyboard controls
    document.addEventListener('keydown', (event) => {
      this.keys[event.code] = true;
    });

    document.addEventListener('keyup', (event) => {
      this.keys[event.code] = false;
    });

    // Mouse controls
    document.addEventListener('click', () => {
      if (!this.isMouseLocked) {
        document.body.requestPointerLock();
      }
    });

    document.addEventListener('pointerlockchange', () => {
      this.isMouseLocked = document.pointerLockElement === document.body;
    });

    document.addEventListener('mousemove', (event) => {
      if (this.isMouseLocked) {
        this.mouse.x = event.movementX * this.mouseSensitivity;
        this.mouse.y = event.movementY * this.mouseSensitivity;
        this.updateMouseLook();
      }
    });
  }

  private updateMouseLook(): void {
    // Store current rotation order
    const euler = new THREE.Euler(0, 0, 0, 'YXZ');
    euler.setFromQuaternion(this.camera.quaternion);

    // Apply mouse movement
    euler.y -= this.mouse.x;
    euler.x -= this.mouse.y;

    // Clamp vertical rotation to prevent camera flipping
    euler.x = Math.max(-Math.PI / 2, Math.min(Math.PI / 2, euler.x));

    // Apply the rotation
    this.camera.quaternion.setFromEuler(euler);

    this.mouse.x = 0;
    this.mouse.y = 0;
  }

  private terrainMesh: THREE.Mesh | null = null;
  private raycaster = new THREE.Raycaster();
  private wallObjects: THREE.Mesh[] = [];

  public setTerrainMesh(mesh: THREE.Mesh): void {
    this.terrainMesh = mesh;
  }

  public setMazeWalls(walls: THREE.Mesh[]): void {
    this.wallObjects = walls;
  }

  private getTerrainHeightAt(x: number, z: number): number {
    if (!this.terrainMesh) {
      // Fallback to flat terrain if no terrain mesh is available
      return 0;
    }

    // Use raycasting to find the terrain height at the given position
    this.raycaster.set(new THREE.Vector3(x, 100, z), new THREE.Vector3(0, -1, 0));
    const intersects = this.raycaster.intersectObject(this.terrainMesh, true);

    if (intersects.length > 0) {
      return intersects[0].point.y;
    }

    // Fallback to 0 if no intersection found
    return 0;
  }

  private checkWallCollision(newPosition: THREE.Vector3): boolean {
    if (this.wallObjects.length === 0) return false;

    const playerRadius = 1; // Player collision radius
    const playerHeight = 2; // Player height

    // Check collision with each wall
    for (const wall of this.wallObjects) {
      // Get wall bounding box
      const box = new THREE.Box3().setFromObject(wall);

      // Expand the box by player radius for collision detection
      box.expandByScalar(playerRadius);

      // Check if player position intersects with expanded wall box
      if (box.containsPoint(new THREE.Vector3(newPosition.x, newPosition.y, newPosition.z))) {
        return true;
      }
    }

    return false;
  }

  public update(deltaTime: number): void {
    // Handle movement input
    this.direction.set(0, 0, 0);

    if (this.keys['KeyW'] || this.keys['ArrowUp']) {
      this.direction.z = -1;
    }
    if (this.keys['KeyS'] || this.keys['ArrowDown']) {
      this.direction.z = 1;
    }
    if (this.keys['KeyA'] || this.keys['ArrowLeft']) {
      this.direction.x = -1;
    }
    if (this.keys['KeyD'] || this.keys['ArrowRight']) {
      this.direction.x = 1;
    }

    // Normalize direction if there's any movement
    if (this.direction.length() > 0) {
      this.direction.normalize();

      // Get camera's forward and right vectors
      const forward = new THREE.Vector3();
      const right = new THREE.Vector3();

      // Extract forward and right vectors from camera's world matrix
      this.camera.getWorldDirection(forward);
      forward.y = 0;
      forward.normalize();

      // Right vector is perpendicular to forward
      right.crossVectors(forward, new THREE.Vector3(0, 1, 0));

      // Calculate final movement direction
      const moveDirection = new THREE.Vector3();
      moveDirection.addScaledVector(forward, -this.direction.z);
      moveDirection.addScaledVector(right, this.direction.x);

      this.direction.copy(moveDirection);
    }

    // Apply movement with wall collision detection
    const moveVector = this.direction.clone().multiplyScalar(this.moveSpeed * deltaTime);
    moveVector.y = 0; // Don't apply movement in Y direction

    // Store current position and calculate new position
    const currentPosition = this.camera.position.clone();
    const newPosition = currentPosition.clone();

    // Test X movement
    newPosition.x += moveVector.x;
    if (!this.checkWallCollision(newPosition)) {
      this.velocity.x = moveVector.x;
    } else {
      this.velocity.x = 0;
      newPosition.x = currentPosition.x; // Reset X position
    }

    // Test Z movement
    newPosition.z += moveVector.z;
    if (!this.checkWallCollision(newPosition)) {
      this.velocity.z = moveVector.z;
    } else {
      this.velocity.z = 0;
      newPosition.z = currentPosition.z; // Reset Z position
    }

    // Handle jumping
    if ((this.keys['Space'] || this.keys['KeySpace']) && this.isGrounded) {
      this.velocity.y = this.jumpSpeed;
      this.isGrounded = false;
    }

    // Apply gravity
    this.velocity.y += this.gravity * deltaTime;

    // Update position
    this.camera.position.add(this.velocity.clone().multiplyScalar(deltaTime));

    // Check terrain collision
    const terrainHeight = this.getTerrainHeightAt(this.camera.position.x, this.camera.position.z);
    const playerHeight = 2; // Camera height above ground

    if (this.camera.position.y <= terrainHeight + playerHeight) {
      this.camera.position.y = terrainHeight + playerHeight;
      this.velocity.y = 0;
      this.isGrounded = true;
    }

    // Keep player within bounds
    const maxDistance = 90;
    if (Math.abs(this.camera.position.x) > maxDistance) {
      this.camera.position.x = Math.sign(this.camera.position.x) * maxDistance;
    }
    if (Math.abs(this.camera.position.z) > maxDistance) {
      this.camera.position.z = Math.sign(this.camera.position.z) * maxDistance;
    }
  }
}