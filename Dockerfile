# Platform linux/amd64, not wasip1/wasm: kubelet pulls for the host platform
# (RuntimeClassInImageCriAPI is alpha), so a wasm-platform image never
# unpacks. The runwasi wasmtime shim runs the component regardless.
FROM scratch
COPY antree-server.wasm /antree-server.wasm
ENTRYPOINT ["/antree-server.wasm"]
