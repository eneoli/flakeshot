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

        meta = {
          description = "A screenshot tool for wayland and x11!";
          homepage = "https://github.com/eneoli/flakeshot/";
          license = lib.licenses.gpl2;
        };
      };
    in
    {
      apps = forAllSystems
        (pkgs:
          let
            flakeshotPkg = pkgs.callPackage mkFlakeshot { };
          in
          {
            default = {
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
            packages = with pkgs; [
              pkg-config
              patchelf
              gtk3
              pango
              cairo
              glib
              gdk-pixbuf
              libappindicator
            ] ++ [ toolchain ];

            CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER = "${pkgs.llvmPackages.clangUseLLVM}/bin/clang";
            CARGO_ENCODED_RUSTFLAGS = "-Clink-arg=-fuse-ld=${pkgs.mold}/bin/mold";

            LD_LIBRARY_PATH = "$LD_LIBRARY_PATH:${builtins.toString (pkgs.lib.makeLibraryPath packages)} ";
          };
      });
    };
}

