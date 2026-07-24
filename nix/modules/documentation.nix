{ inputs, flake-parts-lib, ... }:
{
  imports = [
    inputs.flake-parts-website.flakeModules.empty-site
  ];
  options.perSystem = flake-parts-lib.mkPerSystemOption
    ({ lib, pkgs, config, ... }:
      let
        mdBookProject = pkgs.stdenv.mkDerivation {
          name = "mdBook-project";
          buildInputs = [ pkgs.mdbook ];
          src = ./.;
          buildPhase = ''
            mkdir -p $out
            cd $out
            mdbook init --force --title "${config.render.inputs.tms-portal.title}"
            cat <<EOF >src/SUMMARY.md
              # Summary

              - [Options](./options.md)
            EOF
            cp ${config.packages.generated-docs-tms-portal}/options.md src
            rm src/chapter_1.md
            mdbook build
          '';
        };
        serve = pkgs.writeShellApplication {
          name = "docs-serve";
          runtimeInputs = [
            pkgs.python3
            pkgs.coreutils
            pkgs.xdg-utils
            config.shell-utils.findPort
            config.shell-utils.display
          ];
          text = ''
            cd ${mdBookProject}/book
            PORT="$(find-port)"
            trap 'kill $(jobs -p) && echo "Documentation server stopped"' EXIT
            python3 -m http.server "$PORT" &
            display "Documentation available at http://localhost:$PORT"
            xdg-open http://localhost:"$PORT"
            sleep infinity
          '';
        };
      in
      {
        options = {
          documentation = {
            mdBookProject = lib.mkOption {
              type = lib.types.package;
              default = mdBookProject;
            };
            serve = lib.mkOption {
              type = lib.types.package;
              default = serve;
            };
          };
        };
      });
  config = {
    flake.flakeModule = import ./default.nix;
    perSystem = { config, ... }: {
      packages = {
        inherit (config.documentation) mdBookProject;
        docs-serve = config.documentation.serve;
      };
      render.inputs.tms-portal = {
        flake = inputs.self;
        baseUrl = "${config.tms-portal.git_url}/blob/main";
        intro = ''
          Introduction to TMS Portal.
        '';
        installation = ''
          Installation instructions for TMS Portal.
        '';
        title = "TMS Portal";
      };
    };
  };
}
