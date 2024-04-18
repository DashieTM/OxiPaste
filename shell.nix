{ pkgs ? import <nixpkgs> {} }:

with pkgs;
mkShell {
  nativeBuildInputs = [
    pkg-config
  ];

  buildInputs = [
    dbus
    gtk4
    libadwaita
    gtk4-layer-shell
  ];

}
