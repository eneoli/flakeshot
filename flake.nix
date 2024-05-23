{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs = inputs@{ flake-parts, ... }:
    let
      mkFlakeshot = import ./nix/package.nix;
    in
    flake-parts.lib.mkFlake { inherit inputs; }
      {
        systems = [
          "x86_64-linux"
          "aarch64-linux"
        ];

        perSystem = { self', lib, system, pkgs, config, ... }: {
          _module.args.pkgs = import inputs.nixpkgs {
            inherit system;

            overlays = with inputs; [
              rust-overlay.overlays.default
            ];
          };

          devShells.default =
            let
              flakeshot = pkgs.callPackage mkFlakeshot { };
              rust-toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
            in
            pkgs.mkShell.override
              {
                stdenv = pkgs.stdenvAdapters.useMoldLinker pkgs.clangStdenv;
              }
              {
                packages = with pkgs; [
                  snixembed
                  cargo-udeps
                  stalonetray
                ] ++ [ rust-toolchain ] ++ flakeshot.nativeBuildInputs ++ flakeshot.buildInputs;
              };
        };

        flake.overlays.default = prev: final: {
          flakeshot = prev.callPackage mkFlakeshot { };
        };
      };
}

