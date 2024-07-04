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
        token: Denomination,
        amount: int
    }

    // type CDKCommitment = {
    //     last_cdk_state: CDKState, // TODO: refer to roots
    // }

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
    var l1_state: CdkId -> CDKState // TODO: refer to commitment
    var cdks_state: CdkId -> CDKState
    var agglayer_state: CdkId -> CDKState // TODO: refer to commitment
    var epoch: int

    /// State initialization
    action init = all {
        l1_state' = Map(),
        agglayer_state' = Map(),
        cdks_state' = NETWORKS.mapBy(network => {
            exit_tree: Set(),
            nullifier_tree: Set(),
            balance_tree: Map(MATIC -> 10, USDC -> 10)
        }),
        epoch' = 0,
    }

    /// Upon submission of one bridge exit from one CDK to another
    /// Accumulates it in the source network Exit Tree and decrease the balance
    action user_submit_bridge_exit(origin: CdkId, target: CdkId, token: Denomination, amount: int): bool = {
        val originState = cdks_state.get(origin)
        val bridgeExit = { origin: origin, target: target, token: token, amount: amount }
        all {
            // Should send to your own network
            require(origin != target),
            // State transition
            val newExitTree = originState.exit_tree.union(Set({bridgeExit}))
            val newBalanceTree = originState.balance_tree.put(token, originState.balance_tree.getBalance(token) - amount)
            val newCdkState = originState.with("exit_tree", newExitTree)
                                         .with("balance_tree", newBalanceTree)
            // Write the source network state
            cdks_state' = cdks_state.set(origin, newCdkState),
            // Unchanged state
            l1_state' = l1_state,
            agglayer_state' = agglayer_state,
            epoch' = epoch
        }
    }

    /// Upon import of one bridge exit as target network
    /// Accumulates it in the target network Nullifier Tree and increase the balance
    action user_import_bridge_exit(origin: CdkId, target: CdkId, token: Denomination, amount: int): bool = {
        val targetState = cdks_state.get(target)
        val bridgeExit = { origin: origin, target: target, token: token, amount: amount }
        all {
            // Should not receive from your own network
            require(origin != target),
            // Should exist on the origin network
            require(cdks_state.get(origin).exit_tree.contains(bridgeExit)),
            // Should not be already claimed
            require(not(cdks_state.get(target).nullifier_tree.contains(bridgeExit))),
            // State transition
            val newNullifierTree = targetState.nullifier_tree.union(Set({bridgeExit}))
            val newBalanceTree = targetState.balance_tree.put(token, targetState.balance_tree.getBalance(token) + amount)
            val newCdkState = targetState.with("nullifier_tree", newNullifierTree)
                                         .with("balance_tree", newBalanceTree)
            // Write the target network state
            cdks_state' = cdks_state.set(target, newCdkState),
            // Unchanged state
            l1_state' = l1_state,
            agglayer_state' = agglayer_state,
            epoch' = epoch
        }
    }

    pure def isSubset(old: CDKState, new: CDKState): bool = all {
        old.exit_tree.subseteq(new.exit_tree),
        old.nullifier_tree.subseteq(new.nullifier_tree)
    }

    /// Upon certificate submission from one origin network
    action cdk_submit_certificate(origin: CdkId): bool = {
        val state_on_cdk = cdks_state.get(origin)
        val state_on_agglayer = agglayer_state.get(origin)
        all {
            require(state_on_agglayer.isSubset(state_on_cdk)),
            require(TOKENS.forall(token => state_on_cdk.balance_tree.getBalance(token) >= 0)),
            agglayer_state' = agglayer_state.set(origin, state_on_cdk),
            l1_state' = l1_state,
            cdks_state' = cdks_state,
            epoch' = epoch
        }
    }

    /// Upon new epoch tick on the agglayer
    /// Settlement on the L1 of the latest consistent AggLayer state
    action agglayer_has_new_epoch(): bool = all {
        cdks_state' = cdks_state,
        agglayer_state' = agglayer_state,
        l1_state' = l1_state.keys().mapBy(cdk_id => agglayer_state.get(cdk_id)),
        epoch' = epoch + 1
    }

    /// Defines what are all the possible steps in the state machine
    action step: bool = {
        nondet origin = NETWORKS.oneOf()
        nondet target = NETWORKS.exclude(Set(origin)).oneOf()
        nondet amount = oneOf(1.to(100))
        nondet token = oneOf(TOKENS)
        any {
            user_submit_bridge_exit(origin, target, token, amount),
            user_import_bridge_exit(origin, target, token, amount),
            cdk_submit_certificate(origin),
            agglayer_has_new_epoch
        }
    }

    ///
    /// Invariant and safety properties
    ///

    pure def state_is_non_negative(origin: CdkId, state: CdkId -> CDKState): bool = {
        not(state_is_negative(origin, state))
    }

    pure def state_is_negative(origin: CdkId, state: CdkId -> CDKState): bool = {
        TOKENS.forall(token => state.get(origin).balance_tree.getBalance(token) < 0)
    }

    // CDK can be negative, they just won't be able to validate on the agglayer
    val bad_cdk_state_exists = NETWORKS.forall(network => state_is_non_negative(network, agglayer_state))

    // CDK that are negative should not be able to submit Certificate
    val bad_cdk_are_stuck = NETWORKS.forall(network => {
        state_is_negative(network, cdks_state) implies not(enabled(cdk_submit_certificate(network)))
    })

    // AggLayer and L1 state should always be consistent with the invariant
    val settled_state_is_non_negative = NETWORKS.forall(network => {
        state_is_non_negative(network, l1_state) and state_is_non_negative(network, agglayer_state)
    })

    temporal agglayer_safe = always(settled_state_is_non_negative)
    temporal bad_cdk_always_stuck = always(bad_cdk_are_stuck)
    temporal cdk_can_become_byzantine = eventually(bad_cdk_state_exists)
}
