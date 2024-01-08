{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs = inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];

      perSystem = { self', lib, system, pkgs, config, ... }:
        let
          nativeBuildInputs = with pkgs; [
            pkg-config
            pango
            gdk-pixbuf
          ];

          buildInputs = with pkgs; [
            gtk4
            gtk4-layer-shell
            libadwaita
          ];

          mkFlakeshot = { rustPlatform, lib, ... }: rustPlatform.buildRustPackage rec {
            inherit nativeBuildInputs buildInputs;

            pname = "flakeshot";
            version = "0.0.1";

            src = builtins.path {
              path = ./.;
            };

            cargoLock.lockFile = ./Cargo.lock;

            meta = {
              description = "A screenshot tool for wayland and x11!";
              homepage = "https://github.com/eneoli/flakeshot/";
              license = lib.licenses.gpl2;
              mainProgram = pname;
            };
          };
        in
        {
          _module.args.pkgs = import inputs.nixpkgs {
            inherit system;

            overlays = with inputs; [
              rust-overlay.overlays.default
            ];
          };

          apps.default = {
            type = "app";
            program = lib.meta.getExe self'.packages;
          };

          packages.default = pkgs.callPackage mkFlakeshot { };

          devShells.default =
            let
              rust-toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
            in
            pkgs.mkShell.override
              {
                stdenv = pkgs.stdenvAdapters.useMoldLinker pkgs.clangStdenv;
              }
              {
                packages = [ rust-toolchain ] ++ nativeBuildInputs ++ buildInputs;
              };
        };
    };
}

