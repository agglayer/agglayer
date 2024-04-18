//! Agglayer smart-contract bindings.

pub(crate) mod polygon_rollup_manager {
    use alloy::sol;

    sol!(
        #[sol(rpc, rename_all = "pascalcase")]
        PolygonRollupManager,
        "./src/contracts/polygonrollupmanager.json"
    );
}

pub(crate) mod polygon_zk_evm {
    use alloy::sol;

    sol!(
        #[sol(rpc)]
        PolygonZkEvm,
        "./src/contracts/polygonzkevm.json"
    );
}
