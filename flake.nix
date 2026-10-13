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
          cargo
          rustc
          rust-analyzer
          # the packages we want
        ];
        shellHook = "";
      };
    };
}
