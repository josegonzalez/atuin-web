{
  description = "A web UI for atuin, to browse and search your shell history in a browser";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-darwin"
        "x86_64-linux"
      ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
      pkgsFor = system: nixpkgs.legacyPackages.${system};
    in
    {
      packages = forAllSystems (system: rec {
        atuin-web = (pkgsFor system).callPackage ./nix/package.nix { };
        default = atuin-web;
      });

      overlays.default = final: _prev: {
        atuin-web = final.callPackage ./nix/package.nix { };
      };

      nixosModules.atuin-web = ./nix/module.nix;
      nixosModules.default = self.nixosModules.atuin-web;

      devShells = forAllSystems (
        system:
        let
          pkgs = pkgsFor system;
        in
        {
          default = pkgs.mkShell {
            packages = [
              pkgs.cargo
              pkgs.clippy
              pkgs.rust-analyzer
              pkgs.rustc
              pkgs.rustfmt
            ];
          };
        }
      );

      formatter = forAllSystems (system: (pkgsFor system).nixfmt-tree);
    };
}
