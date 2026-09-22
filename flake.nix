{
  description = "Build Deps for CARDoor";

  inputs = {
    # recent non-unstable nixpkgs. no special intention
    nixpkgs.url = "https://channels.nixos.org/nixpkgs-26.05/nixexprs.tar.zst";
  };

  outputs = inputs: {
    devShell = {pkgs, system}: {
      default = pkgs.mkShell.override {
        # override stdenv stuff here, if need be
      } {
        packages = with pkgs; [
          # the packages we want
        ];
      };
    }
    # set c formatter
    #formatter = ; 
  };
}
