use std::{env, fs, path::Path};

use rcgen::{CertificateParams, ExtendedKeyUsagePurpose as EKU, KeyUsagePurpose as KU, SanType};

fn main() {
    // Step 1: Generate CA certificate.
    let ca_keys = rcgen::KeyPair::generate().expect("CA key generation failed");
    let ca = {
        let mut params = CertificateParams::default();
        params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
        params.key_usages = vec![KU::KeyCertSign, KU::CrlSign];
        params.distinguished_name = {
            let mut dn = rcgen::DistinguishedName::new();
            dn.push(rcgen::DnType::CommonName, "AgglayerDevCA");
            dn
        };

        rcgen::CertifiedIssuer::self_signed(params, &ca_keys)
            .expect("Failed to create CA certificate")
    };

    // Step 2: Generate server certificate signed by CA.
    let server_keys = rcgen::KeyPair::generate().expect("Server key generation failed");
    let server_cert = {
        let mut params = CertificateParams::default();
        params.is_ca = rcgen::IsCa::NoCa;
        params.key_usages = vec![KU::DigitalSignature, KU::KeyEncipherment];
        params.extended_key_usages = vec![EKU::ServerAuth];

        params.distinguished_name = {
            let mut server_dn = rcgen::DistinguishedName::new();
            server_dn.push(rcgen::DnType::CommonName, "localhost");
            server_dn
        };

        params.subject_alt_names = vec![
            SanType::DnsName("localhost".try_into().unwrap()),
            SanType::IpAddress(std::net::Ipv4Addr::LOCALHOST.into()),
            SanType::IpAddress(std::net::Ipv6Addr::LOCALHOST.into()),
        ];

        params
            .signed_by(&server_keys, &ca)
            .expect("Failed to sign server certificate")
    };

    // Step 3: Write all the certificates to files.
    let out_dir = env::var_os("OUT_DIR").expect("OUT_DIR not set");
    let out_dir = Path::new(&out_dir);
    fs::write(out_dir.join("ca.cert.pem"), ca.pem()).expect("Failed to write CA certificate");
    fs::write(out_dir.join("server.key.pem"), server_keys.serialize_pem())
        .expect("Failed to write server key");
    fs::write(out_dir.join("server.cert.pem"), server_cert.pem())
        .expect("Failed to write server certificate");

    println!("cargo:rerun-if-changed=build.rs");
}
