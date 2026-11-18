{
  description = "Build Deps for CARDoor";

  inputs = {
    # recent non-unstable nixpkgs. no special intention
    nixpkgs.url = "https://channels.nixos.org/nixos-unstable/nixexprs.tar.zst";
  };

  outputs =
    { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        buildInputs = [
          # Shared / Flipper Zero tools
          pkgs.sqlite
          pkgs.rust-analyzer
          pkgs.gcc
          pkgs.rustfmt
          pkgs.pkg-config
          pkgs.rustup
          pkgs.udev
          pkgs.cargo-watch

          # GTK4 & Libadwaita C libraries for sensor-client
          pkgs.glib
          pkgs.gtk4
          pkgs.libadwaita
          pkgs.pango
          pkgs.cairo
          pkgs.graphene
          pkgs.gdk-pixbuf
          pkgs.gsettings-desktop-schemas        
        ];
        
        shellHook = ''
          # Prevent Nix from forcing its host RUSTC compiler onto rustup
          unset RUSTC
          export PATH="$HOME/.cargo/bin:$PATH"
          export XDG_DATA_DIRS="${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}:${pkgs.gtk4}/share/gsettings-schemas/${pkgs.gtk4.name}:$XDG_DATA_DIRS" 
        '';
      };
    };
}
