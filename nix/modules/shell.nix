{ ... }:
{
  perSystem =
    { self', inputs', pkgs, config, ... }:
    {
      devShells.default = (config.rust.craneLib.devShell.override {
        mkShell = inputs'.shell-utils.lib.shell;
      }) {
        name = "TMS-Portal-Dev";
        extraInitRc = ''
          alias sudo='\sudo env PATH="$PATH" HOME="$HOME"'
        '';
        # inputsFrom = with config.packages; [
        #   tms-provider
        #   wrapped-tms-provider
        # ];
        packages = [
          pkgs.httpie
          pkgs.jq
          pkgs.postgresql
          self'.packages.postgres-stack
          pkgs.glibcLocalesUtf8
          pkgs.locale
          pkgs.nodejs_24
          inputs'.agenix.packages.default
        ];
      };
    };
}
