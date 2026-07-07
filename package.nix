{ pkg-config, libtorch-bin, rustPlatform, makeWrapper, onnxruntime, vulkan-loader, llvm, lib }:
let
  libtorch = libtorch-bin.override { cudaSupport = true; };
in
rustPlatform.buildRustPackage {
  pname = "snout-cli";
  version = "main";

  src = ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
  };
  cargoBuildFlags = [ "--package" "snout-cli" "-F" "torch-cuda"];

  LIBTORCH = libtorch.dev;
  nativeBuildInputs = [
    pkg-config
    rustPlatform.bindgenHook
    makeWrapper
  ];

  buildInputs = [
    libtorch.dev
  ];

  postFixup = let
    libs = lib.makeLibraryPath [
      llvm
      (onnxruntime.override { cudaSupport = true; })
      vulkan-loader
      libtorch
    ];
  in
    ''
          wrapProgram "$out/bin/snout-cli" \
            --prefix LD_LIBRARY_PATH : "${libs}"
  '';

  meta = {
    description = "A library for snout detection and tracking";
    homepage = "https://github.com/Darksecond/libsnout";
    platforms = lib.platforms.linux;
    mainProgram = "snout-cli";
  };
}
