import * as THREE from 'three';

export class HouseGenerator {
  public generateHouse(): THREE.Group {
    const house = new THREE.Group();

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
    house.add(walls);

    // Roof
    const roofGeometry = new THREE.ConeGeometry(width * 0.8, height * 0.6, 4);
    const roofMaterial = new THREE.MeshLambertMaterial({ color: 0x8b4513 });
    const roof = new THREE.Mesh(roofGeometry, roofMaterial);
    roof.position.y = height + height * 0.3;
    roof.rotation.y = Math.PI / 4;
    roof.castShadow = true;
    house.add(roof);

    // Door
    const doorGeometry = new THREE.BoxGeometry(width * 0.2, height * 0.6, 0.2);
    const doorMaterial = new THREE.MeshLambertMaterial({ color: 0x4a3c28 });
    const door = new THREE.Mesh(doorGeometry, doorMaterial);
    door.position.set(0, height * 0.3, depth / 2 + 0.1);
    house.add(door);

    // Windows
    const windowGeometry = new THREE.BoxGeometry(width * 0.15, height * 0.2, 0.2);
    const windowMaterial = new THREE.MeshLambertMaterial({
      color: 0x87ceeb,
      emissive: 0x87ceeb,
      emissiveIntensity: 0.2
    });

    // Front windows
    const window1 = new THREE.Mesh(windowGeometry, windowMaterial);
    window1.position.set(-width * 0.25, height * 0.6, depth / 2 + 0.1);
    house.add(window1);

    const window2 = new THREE.Mesh(windowGeometry, windowMaterial);
    window2.position.set(width * 0.25, height * 0.6, depth / 2 + 0.1);
    house.add(window2);

    // Side windows
    const window3 = new THREE.Mesh(windowGeometry, windowMaterial);
    window3.position.set(width / 2 + 0.1, height * 0.6, 0);
    window3.rotation.y = Math.PI / 2;
    house.add(window3);

    return house;
  }
}