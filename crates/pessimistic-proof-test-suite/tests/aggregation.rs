use aggregation_proof_core::AggregationPublicValues;
use pessimistic_proof_test_suite::{cert_graph::CertGraphBuilder, AGGREGATION_PROOF_ELF};
use sp1_sdk::{ProverClient, SP1Stdin};

#[test]
fn simple_aggregation() {
    let cert_graph = {
        let mut dag = CertGraphBuilder::default();
        let a1 = dag.add_cert('A');
        let a2 = dag.add_cert('A');
        let a3 = dag.add_cert('A');
        let b1 = dag.add_cert('B');
        let b2 = dag.add_cert('B');

        b1.claims_from(a1, &mut dag);
        b2.claims_from(a3, &mut dag);

        dag.build()
    };

    let aggregation_witness = cert_graph.aggregation_witness();
    //println!("{:#?}", aggregation_witness);

    let mut stdin = SP1Stdin::new();
    stdin.write(&aggregation_witness);

    let client = ProverClient::from_env();

    let (pv, report) = client
        .execute(AGGREGATION_PROOF_ELF, &stdin)
        .deferred_proof_verification(false)
        .run()
        .unwrap();
    let pv_sp1_execute: AggregationPublicValues = AggregationPublicValues::bincode_codec()
        .deserialize(pv.as_slice())
        .unwrap();

    println!("aggregation public values: {:?}", pv_sp1_execute);
}
