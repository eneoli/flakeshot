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
}
