{ flake-parts-lib, ... }: {
  options.perSystem = flake-parts-lib.mkPerSystemOption ({ lib, ... }: { 
    options.tms-portal.frontend = {
        redirect_uri = lib.mkOption {
            type = lib.types.str;
            default = "https://tms-auth-service.tacc.cloud/";
            description = "Redirect URI for Identity Provider";
        };
    };
  });
  config.perSystem = { pkgs, config, ... }:
    let
      myPnpm = pkgs.pnpm_11.override { nodejs-slim = pkgs.nodejs-slim_24; };
      frontend = pkgs.stdenv.mkDerivation (final: {
        name = "frontend";
        pname = final.name;
        src = ./../../client;
        strictDeps = true;
        nativeBuildInputs = with pkgs; [
          nodejs_24
          pnpmConfigHook
          pnpmBuildHook
          makeBinaryWrapper
          myPnpm
        ];
        pnpmDeps = pkgs.fetchPnpmDeps {
          inherit (final) pname name src;
          inherit myPnpm;
          fetcherVersion = 4;
          hash = "sha256-fN07nWD8ZyJHu+OTAzNQquyYH5pmgtMx6poMRUOkazM=";
        };
        VITE_REDIRECT_URI = config.tms-portal.frontend.redirect_uri;
        pnpmBuildScript = "build";
        pnpmBuildFlags = [
          "--mode"
          "production"
        ];
        installPhase = ''
          runHook preInstall
          mkdir $out
          cp -r dist/. $out
          runHook postInstall 
        '';
      });
    in
    {
      packages = {
        inherit frontend;
      };
    };
}
