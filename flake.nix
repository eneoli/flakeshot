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

            overlays = [ (import rust-overlay) ];
          }));
    in
    {
      devShells = forAllSystems (pkgs: {
        default =
          let
            toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          in
          pkgs.mkShell rec {
            packages = with pkgs; [
              toolchain
            ];

            buildInputs = with pkgs; [
              xorg.libX11
              xorg.libXcursor
              xorg.libXrandr

              libxkbcommon
              dbus
            ];

            LD_LIBRARY_PATH = "$LD_LIBRARY_PATH:${builtins.toString (pkgs.lib.makeLibraryPath buildInputs)}";
          };
      });
    };
}
