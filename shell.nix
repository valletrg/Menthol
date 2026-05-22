{
  pkgs ? import <nixpkgs> { },
}:

pkgs.mkShell {
  buildInputs = with pkgs; [
    gtk4
    libadwaita
    glib
    pkg-config
    cargo
    rustc
    libiconv
  ];
}
