{
  description = "Rust library for face and eye tracking based on project babble";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs = { nixpkgs, ... }:
  let
    inherit (nixpkgs) lib;
    eachSystem = function: lib.genAttrs (
      lib.platforms.linux ++ lib.platforms.darwin
    ) (system:
      function (import nixpkgs {
        inherit system;
      })
    );
  in
  {
    overlays.default = prev: final: {
      libsnout = prev.callPackage ./package.nix {};
    };

    packages = eachSystem (pkgs: {
      default = pkgs.callPackage ./package.nix {};
      });
  };
}
