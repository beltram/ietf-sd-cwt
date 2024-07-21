use sd_cwt::IssuerPrivateKey;

#[test]
fn should_build_sd_cwt() {
    let issuer_key = IssuerPrivateKey::generate();
    let sd_cwt = issuer_key.sign(input).unwrap();

}