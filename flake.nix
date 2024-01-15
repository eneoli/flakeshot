{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs = inputs@{ flake-parts, ... }:
    let
      mkFlakeshot =
        { rustPlatform
        , pkgs
        , lib
        , pkg-config
        , pango
        , gdk-pixbuf
        , gtk4
        , gtk4-layer-shell
        , libadwaita
        , wrapGAppsHook4
        , glib
        , ...
        }: rustPlatform.buildRustPackage.override
          {
            stdenv = pkgs.stdenvAdapters.useMoldLinker pkgs.clangStdenv;
          }
          rec {

            nativeBuildInputs = [
              pkg-config
              pango
              gdk-pixbuf
              glib
              wrapGAppsHook4
            ];

            buildInputs = [
              gtk4
              gtk4-layer-shell
              libadwaita
            ];

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
                  stalonetray
                ] ++ [ rust-toolchain ] ++ flakeshot.nativeBuildInputs ++ flakeshot.buildInputs;
              };
        };

        flake.overlays.default = prev: final: {
          flakeshot = prev.callPackage mkFlakeshot { };
        };
      };
}

