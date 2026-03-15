import * as THREE from "three/webgpu";

// Mystral provides a global canvas wired to the WebGPU surface.
// In a browser, we'd create our own — but here we must use theirs.
declare const canvas: HTMLCanvasElement;

export class Renderer {
  scene: THREE.Scene;
  camera: THREE.PerspectiveCamera;
  renderer: THREE.WebGPURenderer;
  raycaster: THREE.Raycaster;
  mouse: THREE.Vector2;
  width: number;
  height: number;

  private isDragging = false;
  private prevMouse = { x: 0, y: 0 };
  private spherical = { radius: 300, phi: Math.PI / 3, theta: 0 };
  private target = new THREE.Vector3(0, 0, 0);

  private constructor(gpuRenderer: THREE.WebGPURenderer, w: number, h: number) {
    this.width = w;
    this.height = h;

    this.scene = new THREE.Scene();
    this.scene.background = new THREE.Color(0x0a0a1a);

    this.camera = new THREE.PerspectiveCamera(60, w / h, 1, 5000);
    this.updateCameraPosition();

    this.renderer = gpuRenderer;

    this.raycaster = new THREE.Raycaster();
    this.mouse = new THREE.Vector2();

    // Lighting
    const ambient = new THREE.AmbientLight(0x334466, 1.5);
    this.scene.add(ambient);

    const point = new THREE.PointLight(0xffffff, 2, 1000);
    point.position.set(100, 200, 150);
    this.scene.add(point);

    const point2 = new THREE.PointLight(0x4488ff, 1, 800);
    point2.position.set(-150, -100, -100);
    this.scene.add(point2);

    // Grid helper for spatial reference
    const grid = new THREE.GridHelper(400, 20, 0x222244, 0x111133);
    grid.position.y = -100;
    this.scene.add(grid);

    this.setupControls();
  }

  static async create(): Promise<Renderer> {
    const w = canvas.width || 1280;
    const h = canvas.height || 720;

    console.log(`[wiregraph] init renderer ${w}x${h}`);

    const gpuRenderer = new THREE.WebGPURenderer({
      canvas: canvas,
      antialias: false,
    });
    await gpuRenderer.init();

    gpuRenderer.setSize(w, h, false);
    gpuRenderer.setPixelRatio(1);

    console.log("[wiregraph] WebGPU ready");

    return new Renderer(gpuRenderer, w, h);
  }

  private setupControls(): void {
    canvas.addEventListener("mousedown", (e: MouseEvent) => {
      this.isDragging = true;
      this.prevMouse = { x: e.clientX, y: e.clientY };
    });

    canvas.addEventListener("mousemove", (e: MouseEvent) => {
      this.mouse.x = (e.clientX / this.width) * 2 - 1;
      this.mouse.y = -(e.clientY / this.height) * 2 + 1;

      if (!this.isDragging) return;

      const dx = e.clientX - this.prevMouse.x;
      const dy = e.clientY - this.prevMouse.y;
      this.prevMouse = { x: e.clientX, y: e.clientY };

      this.spherical.theta -= dx * 0.005;
      this.spherical.phi = Math.max(0.1, Math.min(Math.PI - 0.1, this.spherical.phi - dy * 0.005));
      this.updateCameraPosition();
    });

    canvas.addEventListener("mouseup", () => {
      this.isDragging = false;
    });

    canvas.addEventListener("wheel", (e: WheelEvent) => {
      this.spherical.radius = Math.max(50, Math.min(2000, this.spherical.radius + e.deltaY * 0.5));
      this.updateCameraPosition();
    });
  }

  private updateCameraPosition(): void {
    const { radius, phi, theta } = this.spherical;
    this.camera.position.set(
      this.target.x + radius * Math.sin(phi) * Math.cos(theta),
      this.target.y + radius * Math.cos(phi),
      this.target.z + radius * Math.sin(phi) * Math.sin(theta),
    );
    this.camera.lookAt(this.target);
  }

  resetCamera(): void {
    this.spherical = { radius: 300, phi: Math.PI / 3, theta: 0 };
    this.target.set(0, 0, 0);
    this.updateCameraPosition();
  }

  render(): void {
    this.renderer.render(this.scene, this.camera);
  }
}
