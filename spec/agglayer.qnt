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


    // // hash on identity
    // pure def Id(v) = v
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
    var agglayer_state: CdkId -> CDKState // TODO: refer to commitment
    var cdks_state: CdkId -> CDKState
    var bridge_service: CdkId -> Set[BridgeExit] // Target -> importable bridge exits

    var epoch: int

    /// State initialization
    action init = all {
        l1_state' = NETWORKS.mapBy(network => {
            exit_tree: Set(),
            nullifier_tree: Set(),
            balance_tree: Map(MATIC -> 0, USDC -> 0)
        }),
        agglayer_state' = NETWORKS.mapBy(network => {
            exit_tree: Set(),
            nullifier_tree: Set(),
            balance_tree: Map(MATIC -> 0, USDC -> 0)
        }),
        cdks_state' = NETWORKS.mapBy(network => {
            exit_tree: Set(),
            nullifier_tree: Set(),
            balance_tree: Map(MATIC -> 10, USDC -> 10)
        }),
        bridge_service' = NETWORKS.mapBy(network => Set()),
        epoch' = 0,
    }

    /// Upon submission of one bridge exit from one CDK to another
    /// Accumulates it in the source network Exit Tree and decrease the balance
    action user_submit_bridge_exit(is_byzantine: bool): bool = {
        nondet origin = NETWORKS.oneOf()
        nondet target = NETWORKS.exclude(Set(origin)).oneOf()
        nondet amount = oneOf(1.to(100))
        nondet token = oneOf(TOKENS)
        val originState = cdks_state.get(origin)
        val bridgeExit = { origin: origin, target: target, token: token, amount: amount }
        val newExitTree = originState.exit_tree.union(Set({bridgeExit}))
        val newBalance = originState.balance_tree.getBalance(token) - amount
        all {
            // Should send to your own network
            require(origin != target),
            // Is byzantine or should be non negative
            require(is_byzantine or newBalance >= 0),
            // State transition
            val newBalanceTree = originState.balance_tree.put(token, newBalance)
            val newCdkState = originState.with("exit_tree", newExitTree)
                                         .with("balance_tree", newBalanceTree)
            // Write the source network state
            cdks_state' = cdks_state.set(origin, newCdkState),
            // Unchanged state
            l1_state' = l1_state,
            agglayer_state' = agglayer_state,
            epoch' = epoch,
            bridge_service' = bridge_service
        }
    }

    /// Upon import of one bridge exit as target network
    /// Accumulates it in the target network Nullifier Tree and increase the balance
    action user_import_bridge_exit(is_byzantine: bool): bool = {
        nondet network = NETWORKS.oneOf()
        nondet bridgeExit = bridge_service.get(network).oneOf()
        val targetState = cdks_state.get(bridgeExit.target)
        val origin = bridgeExit.origin
        val target = bridgeExit.target
        val token = bridgeExit.token
        all {
            // Should not receive from your own network
            require(origin != target),
            // Should be byzantine or should exist on the origin network
            require(is_byzantine or l1_state.get(origin).exit_tree.contains(bridgeExit)),
            // Should be byzantine or not be already claimed
            require(is_byzantine or not(l1_state.get(target).nullifier_tree.contains(bridgeExit))),
            // State transition
            val newNullifierTree = targetState.nullifier_tree.union(Set({bridgeExit}))
            val newBalance = targetState.balance_tree.getBalance(token) + bridgeExit.amount
            val newBalanceTree = targetState.balance_tree.put(token, newBalance)
            val newCdkState = targetState.with("nullifier_tree", newNullifierTree)
                                         .with("balance_tree", newBalanceTree)
            // Write the target network state
            cdks_state' = cdks_state.set(target, newCdkState),
            bridge_service' = bridge_service.setBy(target, bridge_exits => bridge_exits.exclude(Set({bridgeExit}))),
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

    pure def pessimistic_proof(origin: CdkId, state_on_cdk: CDKState, state_on_agglayer: CDKState): bool = all {
        // On top of settled state
        require(state_on_agglayer.isSubset(state_on_cdk)),
        // Non negative balance
        require(TOKENS.forall(token => state_on_cdk.balance_tree.getBalance(token) >= 0)),
    }

    /// Upon certificate submission from one origin network
    action cdk_submit_certificate(origin: CdkId): bool = {
        val state_on_cdk = cdks_state.get(origin)
        val state_on_agglayer = agglayer_state.get(origin)
        all {
            require(pessimistic_proof(origin, state_on_cdk, state_on_agglayer)),
            // Pessimistic proof statem
            agglayer_state' = agglayer_state.set(origin, state_on_cdk),
            l1_state' = l1_state,
            cdks_state' = cdks_state,
            epoch' = epoch,
            bridge_service' = bridge_service
        }
    }

    /// Upon new epoch tick on the agglayer
    /// Settlement on the L1 of the latest consistent AggLayer state
    action agglayer_has_new_epoch(): bool = all {
        cdks_state' = cdks_state,
        agglayer_state' = agglayer_state,
        l1_state' = l1_state.keys().mapBy(cdk_id => agglayer_state.get(cdk_id)),
        epoch' = epoch + 1,
        bridge_service' = bridge_service
    }

    /// Defines what are all the possible steps in the state machine
    action step: bool = {
        nondet is_byzantine = oneOf(Set(true, false))
        any {
            user_submit_bridge_exit(is_byzantine),
            user_import_bridge_exit(is_byzantine),
            nondet cert_submitter = NETWORKS.oneOf()
            cdk_submit_certificate(cert_submitter),
            agglayer_has_new_epoch
        }
    }

    ///
    /// Invariant
    ///
    pure def state_is_non_negative(origin: CdkId, state: CdkId -> CDKState): bool = {
        not(state_is_negative(origin, state))
    }

    pure def state_is_negative(origin: CdkId, state: CdkId -> CDKState): bool = {
        TOKENS.exists(token => state.get(origin).balance_tree.getBalance(token) < 0)
    }

    // CDK can be negative, they just won't be able to validate on the agglayer
    val bad_cdk_state_exists = NETWORKS.exists(network => state_is_negative(network, cdks_state))

    // CDK with negative balances should be unable to submit Certificate
    val bad_cdk_are_stuck = NETWORKS.forall(network => {
        state_is_negative(network, cdks_state) implies not(enabled(cdk_submit_certificate(network)))
    })

    // AggLayer and L1 state should always be consistent
    val settled_state_is_non_negative = NETWORKS.forall(network => {
        state_is_non_negative(network, l1_state) and state_is_non_negative(network, agglayer_state)
    })
}
