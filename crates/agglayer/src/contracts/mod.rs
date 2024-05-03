//! Agglayer smart-contract bindings.

pub(crate) mod polygon_rollup_manager {
    use ethers::contract::abigen;

    abigen!(
        PolygonRollupManager,
        "./src/contracts/polygonrollupmanager.json",
    );
}

pub(crate) mod polygon_zk_evm {
    use ethers::contract::abigen;

    abigen!(PolygonZkEvm, "./src/contracts/polygonzkevm.json",);
}
