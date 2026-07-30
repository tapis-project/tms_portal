{ ... }: {
  options = { };
  config.perSystem = { pkgs, ... }:
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
