{ lib
, fetchFromGitHub
, rustPlatform
, pkg-config
, openssl
}:

rustPlatform.buildRustPackage {
  pname = "nx";
  version = "0.1.0";

  src = fetchFromGitHub {
    owner = "pawprnt";
    repo = "nx";
    rev = "51cc4f3";
    hash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
  };

  cargoHash = "sha256-MX+RLbn4C0lxpWoAgS6AL67jFHiwpWMVbt8ykiy27Gs=";

  nativeBuildInputs = [ pkg-config ];
  buildInputs = [ openssl ];

  meta = with lib; {
    description = "A nix helper CLI";
    homepage = "https://github.com/pawprnt/nx";
    license = licenses.mit;
    mainProgram = "nx";
  };
}
