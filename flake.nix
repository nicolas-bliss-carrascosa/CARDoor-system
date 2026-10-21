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
        buildInputs = with pkgs; [
          sqlite
          rust-analyzer
          gcc
          pkg-config
          rustup
          # We very specifically dont want cargo or rustc because they cause problems with compiling to the flipper zero target
        ];
        
        shellHook = ''
          # Prevent Nix from forcing its host RUSTC compiler onto rustup
          unset RUSTC
          export PATH="$HOME/.cargo/bin:$PATH"
        '';
      };
    };
}
