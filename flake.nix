{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { nixpkgs, rust-overlay, ... }:
    let
      forAllSystems = function:
        nixpkgs.lib.genAttrs [
          "x86_64-linux"
        ]
          (system: function (import nixpkgs {
            inherit system;

            overlays = [ rust-overlay.overlays.default ];
          }));

      mkFlakeshot = { rustPlatform, lib, ... }: rustPlatform.buildRustPackage {
        pname = "flakeshot";
        version = "0.0.1";

        src = ./.;
        cargoLock.lockFile = ./Cargo.lock;

        cargoSha256 = "";

        meta = {
          description = "A screenshot tool for wayland and xorg!";
          homepage = "https://github.com/eneoli/flakeshot/";
          license = lib.licenses.mit;
        };
      };
    in
    {
      apps = forAllSystems
        (pkgs:
          let
            flakeshotPkg = pkgs.callPackage mkFlakeshot { };
          in
          rec {
            default = flakeshot;

            flakeshot = {
              type = "app";
              program = "${flakeshotPkg}/bin/flakeshot";
            };
          });

      overlays.default = final: prev: {
        flakeshot = prev.callPackage mkFlakeshot { };
      };

      devShells = forAllSystems (pkgs: {
        default =
          let
            toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          in
          pkgs.mkShell rec {
            packages = [
              toolchain
            ];

            buildInputs = with pkgs; [
              xorg.libX11
              xorg.libXcursor
              xorg.libXrandr

              libxkbcommon
              dbus
            ];

            LD_LIBRARY_PATH = "$LD_LIBRARY_PATH:${builtins.toString (pkgs.lib.makeLibraryPath buildInputs)} ";
          };
      });
    };
}

