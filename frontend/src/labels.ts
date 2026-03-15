import * as THREE from "three/webgpu";

// Billboard sprite with text label.
// Canvas2D → texture → SpriteMaterial.
export function createLabel(text: string, color: string = "#aaccff"): THREE.Sprite {
  const w = 256;
  const h = 64;
  const c = document.createElement("canvas") as HTMLCanvasElement;
  c.width = w;
  c.height = h;
  const ctx = c.getContext("2d")!;

  ctx.clearRect(0, 0, w, h);
  ctx.fillStyle = color;
  ctx.font = "bold 26px monospace";
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  ctx.fillText(text, w / 2, h / 2);

  const tex = new THREE.CanvasTexture(c);
  tex.needsUpdate = true;

  const mat = new THREE.SpriteMaterial({
    map: tex,
    transparent: true,
    depthTest: false,
  });

  const sprite = new THREE.Sprite(mat);
  sprite.scale.set(28, 7, 1);
  return sprite;
}
