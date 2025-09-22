import * as THREE from 'three';

export class Player {
  private camera: THREE.PerspectiveCamera;
  private scene: THREE.Scene;
  private velocity: THREE.Vector3 = new THREE.Vector3();
  private direction: THREE.Vector3 = new THREE.Vector3();
  private keys: { [key: string]: boolean } = {};
  private mouse: { x: number; y: number } = { x: 0, y: 0 };
  private isMouseLocked: boolean = false;

  private moveSpeed: number = 500;
  private jumpSpeed: number = 20;
  private gravity: number = -35;
  private isGrounded: boolean = true;

  constructor(camera: THREE.PerspectiveCamera, scene: THREE.Scene) {
    this.camera = camera;
    this.scene = scene;

    // Set initial position
    this.camera.position.set(-30, 10, 0);

    this.initControls();
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
        this.mouse.x = event.movementX * 0.002;
        this.mouse.y = event.movementY * 0.002;
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

  private getTerrainHeightAt(x: number, z: number): number {
    // Calculate terrain height using the same formula as TerrainGenerator
    const terrainSize = 200;
    const fx = (x / terrainSize + 0.5);
    const fz = (z / terrainSize + 0.5);

    const height =
      Math.sin(fx * 8) * Math.sin(fz * 8) * 3 +
      Math.sin(fx * 16) * Math.sin(fz * 16) * 1.5 +
      Math.sin(fx * 4) * Math.sin(fz * 4) * 5;

    return height;
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

    // Apply movement
    const moveVector = this.direction.clone().multiplyScalar(this.moveSpeed * deltaTime);
    moveVector.y = 0; // Don't apply movement in Y direction

    this.velocity.x = moveVector.x;
    this.velocity.z = moveVector.z;

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