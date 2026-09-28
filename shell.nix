{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  packages = with pkgs; [ cargo rustc rustfmt clippy pkg-config python3 ];
  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (with pkgs; [
    wayland libxkbcommon libx11 libxcursor libxrandr libxi libxcb
  ]);
}
