load("antree.wasm");

// macroquad sizes the canvas at start but keeps its 1280x720 config until a
// window resize event, which a phone never sends. Pass the real size once.
const sync = setInterval(() => {
  if (window.wasm_exports) {
    clearInterval(sync);
    wasm_exports.resize(canvas.width, canvas.height);
  }
}, 50);
