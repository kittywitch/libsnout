{ pkg-config, libtorch-bin, rustPlatform, makeWrapper, onnxruntime, vulkan-loader, llvm, lib }:

rustPlatform.buildRustPackage {
  pname = "snout-cli";
  version = "main";

  src = ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
  };
  cargoBuildFlags = [ "--package" "snout-cli" "-F" "torch-cuda"];

  nativeBuildInputs = [
    pkg-config
    rustPlatform.bindgenHook
    makeWrapper
    libtorch-bin
  ];

  postFixup = let
    libs = lib.makeLibraryPath [
      llvm
      (onnxruntime.override { cudaSupport = true; })
      vulkan-loader
      libtorch-bin
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
