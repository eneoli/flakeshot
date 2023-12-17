{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    systems.url = "github:nix-systems/default-linux";
  };

  outputs = { self, nixpkgs, rust-overlay, systems, ... }:
    let
      eachSystem = nixpkgs.lib.genAttrs (import systems);

      mkFlakeshot = { rustPlatform, lib, ... }: rustPlatform.buildRustPackage rec {
        pname = "flakeshot";
        version = "0.0.1";

        src = builtins.path {
          path = ./.;
          name = pname;
        };
        cargoLock.lockFile = ./Cargo.lock;

        meta = {
          description = "A screenshot tool for wayland and x11!";
          homepage = "https://github.com/eneoli/flakeshot/";
          license = lib.licenses.gpl2;
        };
      };
    in
    {
      apps = eachSystem (system: {
        default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/flakeshot";
        };
      });

      packages = eachSystem (system:
        let
          pkgs = import nixpkgs {
            inherit system;
          };
        in
        {
          default = pkgs.callPackage mkFlakeshot { };
        });

      overlays.default = final: prev: {
        flakeshot = prev.callPackage mkFlakeshot { };
      };

      devShells = eachSystem (system:
        let
          pkgs = import nixpkgs {
            inherit system;

            overlays = [ rust-overlay.overlays.default ];
          };

          toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        in
        {
          default = pkgs.mkShell.override
            {
              stdenv = pkgs.stdenvAdapters.useMoldLinker pkgs.clangStdenv;
            }
            {
              packages = [ toolchain ];
            };
        });
    };
}

