# This flake builds the TMS Portal.

{
  description = "A Nix Flake for the TMS Portal";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    simple-flake.url = "github:waltermoreira/simple-flake";
    shell-utils.url = "github:waltermoreira/shell-utils";
    crane.url = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-parts-website.url = "github:hercules-ci/flake.parts-website";
    process-compose-flake.url = "github:Platonic-Systems/process-compose-flake";
    services-flake.url = "github:juspay/services-flake";
  };

  outputs = inputs@{ simple-flake, ... }:
    simple-flake.lib.mkFlake { inherit inputs; }
      {
        imports = [
          ./nix/modules
          inputs.process-compose-flake.flakeModule
          inputs.shell-utils.flakeModule
          ./nix/modules/documentation.nix
        ];
        config = {
          debug = true;
          systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];
          perSystem = {
            imports = [
              ./nix/config.nix
            ];
          };
        };
      };
}
