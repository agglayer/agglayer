/**
 * A specification for the centralized AggLayer
 */
module agglayer {
    import base.* from "./base"
    import bank.* from "./bank"
    import denomTrace.* from "./denomTrace"

    ///
    /// Spells
    ///
    def require(cond: bool): bool = cond

    ///
    /// Types
    ///
    type CdkId = str

    type Certificate = {
        network_id: CdkId,
        bridge_exits: Set[BridgeExit],
        imported_bridge_exits: Set[BridgeExit]
    }

    type BridgeExit = {
        origin: CdkId,
        target: CdkId,
        denom: Denomination,
        amount: int
    }

    type CDKCommitment = {
        last_cdk_state: CDKState, // TODO: refer to roots
    }

    type CDKState = {
        exit_tree: Set[BridgeExit],
        nullifier_tree: Set[BridgeExit],
        balance_tree: TokenBalances
    }

    ///
    /// Constants
    ///
    pure val USDC = toDenom("usdc")
    pure val MATIC = toDenom("matic")
    pure val TOKENS = Set(MATIC, USDC)
    pure val NETWORKS = Set("A", "B", "C", "D")

    ///
    /// Global State
    ///
    var agg_l1_state: CdkId -> CDKCommitment
    var cdks_state: CdkId -> CDKState
    //var agglayer_state

    action init = all {
        agg_l1_state' = Map(),
        cdks_state' = NETWORKS.mapBy(network => {
            exit_tree: Set(),
            nullifier_tree: Set(),
            balance_tree: Map(MATIC -> 100, USDC -> 100)
        })
    }

    /// Send one bridge exit from one CDK to another
    /// Accumulates it in the source network Exit Tree and decrease the balance
    action user_submit_bridge_exit(origin: CdkId, target: CdkId, denom: Denomination, amount: int): bool = {
        val originState = cdks_state.get(origin)
        val bridgeExit = { origin: origin, target: target, denom: denom, amount: amount }
        all {
            // Should send to your own network
            require(origin != target),
            // State transition
            val newExitTree = originState.exit_tree.union(Set({bridgeExit}))
            val newBalanceTree = originState.balance_tree.put(denom, originState.balance_tree.getBalance(denom) - amount)
            val newCdkState = originState.with("exit_tree", newExitTree)
                                         .with("balance_tree", newBalanceTree)
            // Write the source network state
            cdks_state' = cdks_state.set(origin, newCdkState)
        }
    }

    /// Import one bridge exit as target network
    /// Accumulates it in the target network Nullifier Tree and increase the balance
    action user_import_bridge_exit(origin: CdkId, target: CdkId, denom: Denomination, amount: int): bool = {
        val targetState = cdks_state.get(target)
        val bridgeExit = { origin: origin, target: target, denom: denom, amount: amount }
        all {
            // Should not receive from your own network
            require(origin != target),
            // Should exist on the origin network
            require(cdks_state.get(origin).exit_tree.contains(bridgeExit)),
            // Should not be already claimed
            require(not(cdks_state.get(target).nullifier_tree.contains(bridgeExit))),
            // State transition
            val newNullifierTree = targetState.nullifier_tree.union(Set({bridgeExit}))
            val newBalanceTree = targetState.balance_tree.put(denom, targetState.balance_tree.getBalance(denom) + amount)
            val newCdkState = targetState.with("nullifier_tree", newNullifierTree)
                                         .with("balance_tree", newBalanceTree)
            // Write the target network state
            cdks_state' = cdks_state.set(target, newCdkState)
        }
    }

    action step: bool = {
        nondet origin = NETWORKS.oneOf()
        nondet target = NETWORKS.exclude(Set(origin)).oneOf()
        nondet amount = oneOf(1.to(10))
        nondet denom = oneOf(TOKENS)
        any {
            user_submit_bridge_exit(origin, target, denom, amount),
            user_import_bridge_exit(origin, target, denom, amount),
        }
    }
}