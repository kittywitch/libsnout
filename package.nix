{ pkg-config, rustPlatform, makeWrapper, onnxruntime, vulkan-loader, llvm, lib }:

rustPlatform.buildRustPackage {
  pname = "snout-cli";
  version = "main";

  src = ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
  };
  cargoBuildFlags = [ "--package" "snout-cli" ];

  nativeBuildInputs = [
    pkg-config
    rustPlatform.bindgenHook
    makeWrapper
  ];

  postFixup = let
    libs = lib.makeLibraryPath [
      llvm
      (onnxruntime.override { cudaSupport = true; })
      vulkan-loader
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
