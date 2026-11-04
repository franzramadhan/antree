// On-screen keys for touch screens. Each button sends the same keydown and
// keyup a keyboard would, straight to the canvas, so the game needs no
// touch code of its own.
if (matchMedia("(pointer: coarse)").matches) {
  const keys = [
    ["ArrowUp", "▲", "up"], ["ArrowLeft", "◀", "left"],
    ["ArrowRight", "▶", "right"], ["ArrowDown", "▼", "down"],
    ["Enter", "OK", "ok"], ["Escape", "Esc", "back"],
    ["KeyH", "H"], ["KeyI", "I"], ["KeyN", "N"], ["KeyW", "W"],
    ["Tab", "Tab"], ["KeyM", "M"],
  ];
  const pad = document.createElement("div");
  pad.id = "pad";
  const letters = document.createElement("div");
  letters.className = "letters";
  pad.append(letters);
  const send = (type, code) =>
    canvas.dispatchEvent(new KeyboardEvent(type, { code, bubbles: true }));
  for (const [code, label, slot] of keys) {
    const b = document.createElement("button");
    b.textContent = label;
    if (slot) b.className = slot;
    (slot ? pad : letters).append(b);
    let held = false;
    const up = (e) => {
      e.preventDefault();
      if (held) send("keyup", code);
      held = false;
    };
    b.addEventListener("pointerdown", (e) => {
      e.preventDefault();
      b.setPointerCapture(e.pointerId);
      held = true;
      send("keydown", code);
    });
    b.addEventListener("pointerup", up);
    b.addEventListener("pointercancel", up);
  }
  document.body.append(pad);
}
