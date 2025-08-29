use agglayer_types::{
    aggchain_data::CertificateAggchainDataCtx, primitives::U256, Certificate, Digest, Error,
    L1WitnessCtx, LocalNetworkStateData, PessimisticRootInput,
};
use pessimistic_proof::{
    core::{
        commitment::{
            PessimisticRootCommitmentValues, PessimisticRootCommitmentVersion,
            SignatureCommitmentVersion,
        },
        generate_pessimistic_proof, AggchainData, AggchainProof, MultiSignature,
    },
    local_state::LocalNetworkState,
    unified_bridge::TokenInfo,
    NetworkState, ProofError,
};
use pessimistic_proof_test_suite::{
    forest::Forest,
    sample_data::{ETH, USDC},
    PESSIMISTIC_PROOF_ELF,
};
use rand::random;
use rstest::rstest;
use sp1_sdk::{utils, HashableKey, ProverClient, SP1Stdin};
use unified_bridge::Claim;

fn u(x: u64) -> U256 {
    x.try_into().unwrap()
}

struct VersionConsistencyChecker {
    /// Initial state
    initial_state: LocalNetworkStateData,
    /// Certificate
    certificate: Certificate,
    /// Commitment version of the settled PP root, indicates the latest settled
    /// commitment version.
    last_settled_pp_root_version: PessimisticRootCommitmentVersion,
    /// Version used to sign the commitment contained in the Certificate,
    /// indicates the desired next commitment version.
    certificate_signature_version: SignatureCommitmentVersion,
}

fn state_transition(
    certificate_signature_version: SignatureCommitmentVersion,
) -> (LocalNetworkStateData, Certificate) {
    let mut forest = Forest::new(vec![(USDC, u(100)), (ETH, u(200))]);
    let imported_bridge_events = vec![(USDC, u(50)), (ETH, u(100)), (USDC, u(10))];
    let bridge_events = vec![(USDC, u(20)), (ETH, u(50)), (USDC, u(130))];

    let initial_state = forest.state_b.clone();
    let certificate = forest.apply_events_with_version(
        &imported_bridge_events,
        &bridge_events,
        certificate_signature_version,
    );

    (initial_state, certificate)
}

impl VersionConsistencyChecker {
    fn check(&self) -> Result<(), ProofError> {
        let l1_info_root = self.certificate.l1_info_root().unwrap().unwrap_or_default();
        let signer = self
            .certificate
            .retrieve_signer(self.certificate_signature_version)
            .unwrap();

        let agglayer_types::aggchain_proof::AggchainData::ECDSA { signature } =
            self.certificate.aggchain_data
        else {
            panic!("inconsistent test data")
        };

        self.certificate
            .verify_legacy_ecdsa(signer, &signature)
            .unwrap();

        // Previous state settled in L1
        let expected_prev_pp_root = PessimisticRootCommitmentValues {
            balance_root: self.initial_state.balance_tree.root.into(),
            nullifier_root: self.initial_state.nullifier_tree.root.into(),
            ler_leaf_count: self.initial_state.exit_tree.leaf_count(),
            height: self.certificate.height.as_u64(),
            origin_network: self.certificate.network_id,
        }
        .compute_pp_root(self.last_settled_pp_root_version);

        let multi_batch_header = self
            .initial_state
            .make_multi_batch_header(
                &self.certificate,
                L1WitnessCtx {
                    l1_info_root,
                    prev_pessimistic_root: PessimisticRootInput::Fetched(expected_prev_pp_root),
                    aggchain_data_ctx: CertificateAggchainDataCtx::LegacyEcdsa { signer },
                },
            )
            .unwrap();

        let new_state = {
            let mut state = self.initial_state.clone();
            state
                .apply_certificate(
                    &self.certificate,
                    L1WitnessCtx {
                        l1_info_root,
                        prev_pessimistic_root: PessimisticRootInput::Fetched(expected_prev_pp_root),
                        aggchain_data_ctx: CertificateAggchainDataCtx::LegacyEcdsa { signer },
                    },
                )
                .unwrap();
            state
        };

        // New state about to be settled in L1
        let expected_new_pp_root = PessimisticRootCommitmentValues {
            balance_root: new_state.balance_tree.root.into(),
            nullifier_root: new_state.nullifier_tree.root.into(),
            ler_leaf_count: new_state.exit_tree.leaf_count(),
            height: self.certificate.height.as_u64() + 1,
            origin_network: self.certificate.network_id,
        }
        .compute_pp_root(match self.certificate_signature_version {
            SignatureCommitmentVersion::V2 => PessimisticRootCommitmentVersion::V2,
            _ => PessimisticRootCommitmentVersion::V3,
        });

        let (pv, _) =
            generate_pessimistic_proof(self.initial_state.clone().into(), &multi_batch_header)?;

        assert_eq!(expected_prev_pp_root, pv.prev_pessimistic_root);
        assert_eq!(expected_new_pp_root, pv.new_pessimistic_root);

        Ok(())
    }
}

#[rstest]
// pre-migration: from V2 to V2 is ok
#[case(PessimisticRootCommitmentVersion::V2, SignatureCommitmentVersion::V2, Ok(()))]
// migration: from V2 to V3 is ok
#[case(PessimisticRootCommitmentVersion::V2, SignatureCommitmentVersion::V3, Ok(()))]
// post-migration: from V3 to V3 is ok
#[case(PessimisticRootCommitmentVersion::V3, SignatureCommitmentVersion::V3, Ok(()))]
// rollback: from V3 to V2 is forbidden and lead to an inconsistent signed payload error
#[case(
    PessimisticRootCommitmentVersion::V3,
    SignatureCommitmentVersion::V2,
    Err(ProofError::InconsistentSignedPayload)
)]
fn pp_root_migration(
    #[case] prev_version: PessimisticRootCommitmentVersion,
    #[case] new_version: SignatureCommitmentVersion,
    #[case] expected_result: Result<(), ProofError>,
) {
    let (initial_state, certificate) = state_transition(new_version);

    assert_eq!(
        VersionConsistencyChecker {
            certificate_signature_version: new_version,
            last_settled_pp_root_version: prev_version,
            initial_state,
            certificate
        }
        .check(),
        expected_result
    );
}

fn e2e_local_pp_simple_helper(
    initial_balances: impl IntoIterator<Item = (TokenInfo, U256)>,
    imported_events: impl IntoIterator<Item = (TokenInfo, U256)>,
    events: impl IntoIterator<Item = (TokenInfo, U256)>,
) {
    let imported_events = imported_events.into_iter().collect::<Vec<_>>();
    let events = events.into_iter().collect::<Vec<_>>();

    let mut forest = Forest::new(initial_balances);
    let initial_state = forest.state_b.clone();
    let certificate = forest.apply_events(&imported_events, &events);
    let l1_info_root = certificate.l1_info_root().unwrap().unwrap_or_default();
    let multi_batch_header = initial_state
        .make_multi_batch_header(
            &certificate,
            L1WitnessCtx {
                l1_info_root,
                prev_pessimistic_root: PessimisticRootInput::Computed(
                    PessimisticRootCommitmentVersion::V2,
                ),
                aggchain_data_ctx: CertificateAggchainDataCtx::LegacyEcdsa {
                    signer: forest.get_signer(),
                },
            },
        )
        .unwrap();
    generate_pessimistic_proof(initial_state.into(), &multi_batch_header).unwrap();
}

#[test]
fn e2e_local_pp_simple() {
    e2e_local_pp_simple_helper(
        vec![(USDC, u(100)), (ETH, u(200))],
        vec![(USDC, u(50)), (ETH, u(100)), (USDC, u(10))],
        vec![(USDC, u(20)), (ETH, u(50)), (USDC, u(130))],
    )
}

#[test]
fn e2e_local_pp_simple_zero_initial_balances() {
    e2e_local_pp_simple_helper(
        [],
        vec![(USDC, u(50)), (ETH, u(100)), (USDC, u(10))],
        vec![(USDC, u(20)), (ETH, u(50)), (USDC, u(30))],
    )
}

#[test]
fn e2e_local_pp_overflow_attempt() {
    e2e_local_pp_simple_helper(
        [],
        vec![
            (USDC, U256::MAX),
            (USDC, u(3)),
            (ETH, u(100)),
            (USDC, u(10)),
        ],
        vec![(USDC, u(20)), (ETH, u(50)), (USDC, u(30))],
    )
}

#[test]
fn e2e_local_pp_random() {
    let target = u(u64::MAX);
    let upper = u64::MAX / 10;
    let mut forest = Forest::new(vec![(USDC, target), (ETH, target)]);
    // Generate random bridge events such that the sum of the USDC and ETH amounts
    // is less than `target`
    let get_events = || {
        let mut usdc_acc = U256::ZERO;
        let mut eth_acc = U256::ZERO;
        let mut events = Vec::new();
        loop {
            let amount = u(random::<u64>() % upper);
            let token = if random::<u64>() & 1 == 1 { USDC } else { ETH };
            if token == USDC {
                usdc_acc += amount;
                if usdc_acc > target {
                    break;
                }
            } else {
                eth_acc += amount;
                if eth_acc > target {
                    break;
                }
            }
            events.push((token, amount));
        }
        events
    };
    let imported_bridge_events = get_events();
    let bridge_events = get_events();

    let initial_state = forest.state_b.clone();
    let certificate = forest.apply_events(&imported_bridge_events, &bridge_events);

    let l1_info_root = certificate.l1_info_root().unwrap().unwrap_or_default();
    let multi_batch_header = initial_state
        .make_multi_batch_header(
            &certificate,
            L1WitnessCtx {
                l1_info_root,
                prev_pessimistic_root: PessimisticRootInput::Computed(
                    PessimisticRootCommitmentVersion::V2,
                ),
                aggchain_data_ctx: CertificateAggchainDataCtx::LegacyEcdsa {
                    signer: forest.get_signer(),
                },
            },
        )
        .unwrap();

    generate_pessimistic_proof(initial_state.into(), &multi_batch_header).unwrap();
}

#[test]
fn inconsistent_ger() {
    let mut forest = Forest::new(vec![(USDC, u(100)), (ETH, u(200))]);
    let imported_bridge_events = vec![(USDC, u(50)), (ETH, u(100)), (USDC, u(10))];
    let bridge_events = vec![(USDC, u(20)), (ETH, u(50)), (USDC, u(130))];

    let initial_state = forest.state_b.clone();
    let mut certificate = forest.apply_events(&imported_bridge_events, &bridge_events);

    // Change the global exit root
    {
        let Claim::Mainnet(ref mut claim_0) = certificate.imported_bridge_exits[0].claim_data
        else {
            unreachable!("expect from mainnet");
        };

        claim_0.l1_leaf.inner.global_exit_root = Digest::default();
    }

    let l1_info_root = certificate.l1_info_root().unwrap().unwrap_or_default();
    let res = initial_state.make_multi_batch_header(
        &certificate,
        L1WitnessCtx {
            l1_info_root,
            prev_pessimistic_root: PessimisticRootInput::Computed(
                PessimisticRootCommitmentVersion::V2,
            ),
            aggchain_data_ctx: CertificateAggchainDataCtx::LegacyEcdsa {
                signer: forest.get_signer(),
            },
        },
    );

    assert!(matches!(res, Err(Error::InconsistentGlobalExitRoot)))
}

// Same as `e2e_local_pp_simple` with an SP1 proof on top
#[test]
#[ignore]
fn test_sp1_simple() {
    // Setup logging.
    utils::setup_logger();

    let mut forest = Forest::new(vec![(USDC, u(100)), (ETH, u(200))]);
    let imported_bridge_events = vec![(USDC, u(50)), (ETH, u(100)), (USDC, u(10))];
    let bridge_events = vec![(USDC, u(20)), (ETH, u(50)), (USDC, u(130))];

    let initial_state = forest.state_b.clone();
    let (certificate, aggchain_vkey, aggchain_params, aggchain_proof, signature) =
        forest.apply_events_with_aggchain_proof(&imported_bridge_events, &bridge_events);
    let l1_info_root = certificate.l1_info_root().unwrap().unwrap_or_default();

    let mut multi_batch_header = initial_state
        .make_multi_batch_header(
            &certificate,
            L1WitnessCtx {
                l1_info_root,
                prev_pessimistic_root: PessimisticRootInput::Computed(
                    PessimisticRootCommitmentVersion::V2,
                ),
                aggchain_data_ctx: CertificateAggchainDataCtx::LegacyEcdsa {
                    signer: forest.get_signer(),
                },
            },
        )
        .unwrap();

    // Set the aggchain proof to the sp1 variant
    multi_batch_header.aggchain_data = AggchainData::MultisigAndAggchainProof {
        multisig: MultiSignature {
            signatures: vec![(0, signature)],
            expected_signers: vec![forest.get_signer()],
            threshold: 1,
        },
        aggchain_proof: AggchainProof {
            aggchain_params: aggchain_params.into(),
            aggchain_vkey: aggchain_vkey.hash_u32(),
        },
    };

    let initial_state: NetworkState = LocalNetworkState::from(initial_state).into();
    let mut stdin = SP1Stdin::new();
    stdin.write(&initial_state);
    stdin.write(&multi_batch_header);
    stdin.write_proof(
        *aggchain_proof.try_as_compressed().unwrap(),
        aggchain_vkey.vk,
    );

    // Execute the PP within SP1
    let client = ProverClient::from_env();
    let (_public_vals, _report) = client.execute(PESSIMISTIC_PROOF_ELF, &stdin).run().unwrap();
}
