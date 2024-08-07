pub use polygon_rollup_manager::*;
/// This module was auto-generated with ethers-rs Abigen.
/// More information at: <https://github.com/gakonst/ethers-rs>
#[allow(
    clippy::enum_variant_names,
    clippy::too_many_arguments,
    clippy::upper_case_acronyms,
    clippy::type_complexity,
    dead_code,
    non_camel_case_types,
)]
pub mod polygon_rollup_manager {
    #[allow(deprecated)]
    fn __abi() -> ::ethers::core::abi::Abi {
        ::ethers::core::abi::ethabi::Contract {
            constructor: ::core::option::Option::Some(::ethers::core::abi::ethabi::Constructor {
                inputs: ::std::vec![
                    ::ethers::core::abi::ethabi::Param {
                        name: ::std::borrow::ToOwned::to_owned("_globalExitRootManager"),
                        kind: ::ethers::core::abi::ethabi::ParamType::Address,
                        internal_type: ::core::option::Option::Some(
                            ::std::borrow::ToOwned::to_owned(
                                "contract IPolygonZkEVMGlobalExitRootV2",
                            ),
                        ),
                    },
                    ::ethers::core::abi::ethabi::Param {
                        name: ::std::borrow::ToOwned::to_owned("_pol"),
                        kind: ::ethers::core::abi::ethabi::ParamType::Address,
                        internal_type: ::core::option::Option::Some(
                            ::std::borrow::ToOwned::to_owned(
                                "contract IERC20Upgradeable",
                            ),
                        ),
                    },
                    ::ethers::core::abi::ethabi::Param {
                        name: ::std::borrow::ToOwned::to_owned("_bridgeAddress"),
                        kind: ::ethers::core::abi::ethabi::ParamType::Address,
                        internal_type: ::core::option::Option::Some(
                            ::std::borrow::ToOwned::to_owned(
                                "contract IPolygonZkEVMBridge",
                            ),
                        ),
                    },
                ],
            }),
            functions: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("DEFAULT_ADMIN_ROLE"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("DEFAULT_ADMIN_ROLE"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("activateEmergencyState"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "activateEmergencyState",
                            ),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("addExistingRollup"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("addExistingRollup"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("rollupAddress"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "contract IPolygonRollupBase",
                                        ),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("verifier"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("contract IVerifierRollup"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("forkID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("chainID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("genesis"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "rollupCompatibilityID",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint8"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("addNewRollupType"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("addNewRollupType"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "consensusImplementation",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("verifier"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("contract IVerifierRollup"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("forkID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "rollupCompatibilityID",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint8"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("genesis"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("description"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("bridgeAddress"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("bridgeAddress"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "contract IPolygonZkEVMBridge",
                                        ),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("calculateRewardPerBatch"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "calculateRewardPerBatch",
                            ),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("chainIDToRollupID"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("chainIDToRollupID"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("chainID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("rollupID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint32"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("consolidatePendingState"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "consolidatePendingState",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("rollupID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("pendingStateNum"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("createNewRollup"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("createNewRollup"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("rollupTypeID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("chainID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("admin"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("sequencer"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("gasTokenAddress"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("sequencerURL"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("networkName"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("deactivateEmergencyState"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "deactivateEmergencyState",
                            ),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getBatchFee"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("getBatchFee"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getForcedBatchFee"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("getForcedBatchFee"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getInputSnarkBytes"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("getInputSnarkBytes"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("rollupID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("initNumBatch"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("finalNewBatch"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("newLocalExitRoot"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("oldStateRoot"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("newStateRoot"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getLastVerifiedBatch"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "getLastVerifiedBatch",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("rollupID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint32"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getRoleAdmin"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("getRoleAdmin"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("role"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getRollupBatchNumToStateRoot"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "getRollupBatchNumToStateRoot",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("rollupID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("batchNum"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getRollupExitRoot"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("getRollupExitRoot"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getRollupPendingStateTransitions"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "getRollupPendingStateTransitions",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("rollupID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("batchNum"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "struct LegacyZKEVMStateVariables.PendingState",
                                        ),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getRollupSequencedBatches"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "getRollupSequencedBatches",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("rollupID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("batchNum"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "struct LegacyZKEVMStateVariables.SequencedBatchData",
                                        ),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("globalExitRootManager"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "globalExitRootManager",
                            ),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "contract IPolygonZkEVMGlobalExitRootV2",
                                        ),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("grantRole"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("grantRole"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("role"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("account"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("hasRole"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("hasRole"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("role"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("account"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bool"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("initialize"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("initialize"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("trustedAggregator"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "_pendingStateTimeout",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "_trustedAggregatorTimeout",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("admin"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("timelock"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("emergencyCouncil"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("polygonZkEVM"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "contract PolygonZkEVMExistentEtrog",
                                        ),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("zkEVMVerifier"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("contract IVerifierRollup"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("zkEVMForkID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("zkEVMChainID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("isEmergencyState"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("isEmergencyState"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bool"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("isPendingStateConsolidable"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "isPendingStateConsolidable",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("rollupID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("pendingStateNum"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bool"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("lastAggregationTimestamp"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "lastAggregationTimestamp",
                            ),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned(
                        "lastDeactivatedEmergencyStateTimestamp",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "lastDeactivatedEmergencyStateTimestamp",
                            ),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("multiplierBatchFee"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("multiplierBatchFee"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(16usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint16"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("obsoleteRollupType"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("obsoleteRollupType"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("rollupTypeID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint32"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("onSequenceBatches"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("onSequenceBatches"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "newSequencedBatches",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("newAccInputHash"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("overridePendingState"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "overridePendingState",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("rollupID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "initPendingStateNum",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "finalPendingStateNum",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("initNumBatch"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("finalNewBatch"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("newLocalExitRoot"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("newStateRoot"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("proof"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedArray(
                                        ::std::boxed::Box::new(
                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                        ),
                                        24usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32[24]"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("pendingStateTimeout"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "pendingStateTimeout",
                            ),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("pol"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("pol"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "contract IERC20Upgradeable",
                                        ),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned(
                        "proveNonDeterministicPendingState",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "proveNonDeterministicPendingState",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("rollupID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "initPendingStateNum",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "finalPendingStateNum",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("initNumBatch"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("finalNewBatch"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("newLocalExitRoot"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("newStateRoot"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("proof"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedArray(
                                        ::std::boxed::Box::new(
                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                        ),
                                        24usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32[24]"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("renounceRole"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("renounceRole"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("role"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("account"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("revokeRole"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("revokeRole"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("role"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("account"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("rollupAddressToID"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("rollupAddressToID"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("rollupAddress"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("rollupID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint32"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("rollupCount"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("rollupCount"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint32"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("rollupIDToRollupData"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "rollupIDToRollupData",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("rollupID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint32"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("rollupContract"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "contract IPolygonRollupBase",
                                        ),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("chainID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("verifier"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("contract IVerifierRollup"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("forkID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("lastLocalExitRoot"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "lastBatchSequenced",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("lastVerifiedBatch"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("lastPendingState"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "lastPendingStateConsolidated",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "lastVerifiedBatchBeforeUpgrade",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("rollupTypeID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "rollupCompatibilityID",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint8"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("rollupTypeCount"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("rollupTypeCount"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint32"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("rollupTypeMap"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("rollupTypeMap"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("rollupTypeID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint32"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "consensusImplementation",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("verifier"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("contract IVerifierRollup"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("forkID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "rollupCompatibilityID",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint8"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("obsolete"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bool"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("genesis"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("setBatchFee"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("setBatchFee"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("newBatchFee"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("setMultiplierBatchFee"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "setMultiplierBatchFee",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "newMultiplierBatchFee",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(16usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint16"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("setPendingStateTimeout"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "setPendingStateTimeout",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "newPendingStateTimeout",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("setTrustedAggregatorTimeout"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "setTrustedAggregatorTimeout",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "newTrustedAggregatorTimeout",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("setVerifyBatchTimeTarget"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "setVerifyBatchTimeTarget",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "newVerifyBatchTimeTarget",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("totalSequencedBatches"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "totalSequencedBatches",
                            ),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("totalVerifiedBatches"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "totalVerifiedBatches",
                            ),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("trustedAggregatorTimeout"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "trustedAggregatorTimeout",
                            ),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("updateRollup"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("updateRollup"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("rollupContract"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "contract ITransparentUpgradeableProxy",
                                        ),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("newRollupTypeID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("upgradeData"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("verifyBatchTimeTarget"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "verifyBatchTimeTarget",
                            ),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("verifyBatches"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("verifyBatches"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("rollupID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("pendingStateNum"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("initNumBatch"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("finalNewBatch"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("newLocalExitRoot"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("newStateRoot"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("beneficiary"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("proof"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedArray(
                                        ::std::boxed::Box::new(
                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                        ),
                                        24usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32[24]"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("verifyBatchesTrustedAggregator"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "verifyBatchesTrustedAggregator",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("rollupID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("pendingStateNum"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("initNumBatch"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("finalNewBatch"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("newLocalExitRoot"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("newStateRoot"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("beneficiary"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("proof"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedArray(
                                        ::std::boxed::Box::new(
                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                        ),
                                        24usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32[24]"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
            ]),
            events: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("AddExistingRollup"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("AddExistingRollup"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("rollupID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    indexed: true,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("forkID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("rollupAddress"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("chainID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "rollupCompatibilityID",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "lastVerifiedBatchBeforeUpgrade",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("AddNewRollupType"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("AddNewRollupType"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("rollupTypeID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    indexed: true,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "consensusImplementation",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("verifier"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("forkID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "rollupCompatibilityID",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("genesis"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("description"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("ConsolidatePendingState"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned(
                                "ConsolidatePendingState",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("rollupID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    indexed: true,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("numBatch"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("stateRoot"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("exitRoot"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("pendingStateNum"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("CreateNewRollup"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("CreateNewRollup"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("rollupID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    indexed: true,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("rollupTypeID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("rollupAddress"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("chainID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("gasTokenAddress"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("EmergencyStateActivated"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned(
                                "EmergencyStateActivated",
                            ),
                            inputs: ::std::vec![],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("EmergencyStateDeactivated"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned(
                                "EmergencyStateDeactivated",
                            ),
                            inputs: ::std::vec![],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("Initialized"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("Initialized"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("version"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("ObsoleteRollupType"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("ObsoleteRollupType"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("rollupTypeID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    indexed: true,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("OnSequenceBatches"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("OnSequenceBatches"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("rollupID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    indexed: true,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "lastBatchSequenced",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("OverridePendingState"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned(
                                "OverridePendingState",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("rollupID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    indexed: true,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("numBatch"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("stateRoot"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("exitRoot"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("aggregator"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned(
                        "ProveNonDeterministicPendingState",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned(
                                "ProveNonDeterministicPendingState",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("storedStateRoot"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("provedStateRoot"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("RoleAdminChanged"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("RoleAdminChanged"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("role"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    indexed: true,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("previousAdminRole"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    indexed: true,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("newAdminRole"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    indexed: true,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("RoleGranted"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("RoleGranted"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("role"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    indexed: true,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("account"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: true,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("sender"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: true,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("RoleRevoked"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("RoleRevoked"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("role"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    indexed: true,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("account"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: true,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("sender"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: true,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("SetBatchFee"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("SetBatchFee"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("newBatchFee"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("SetMultiplierBatchFee"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned(
                                "SetMultiplierBatchFee",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "newMultiplierBatchFee",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(16usize),
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("SetPendingStateTimeout"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned(
                                "SetPendingStateTimeout",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "newPendingStateTimeout",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("SetTrustedAggregator"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned(
                                "SetTrustedAggregator",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "newTrustedAggregator",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("SetTrustedAggregatorTimeout"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned(
                                "SetTrustedAggregatorTimeout",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "newTrustedAggregatorTimeout",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("SetVerifyBatchTimeTarget"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned(
                                "SetVerifyBatchTimeTarget",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "newVerifyBatchTimeTarget",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("UpdateRollup"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("UpdateRollup"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("rollupID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    indexed: true,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("newRollupTypeID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "lastVerifiedBatchBeforeUpgrade",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("VerifyBatches"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("VerifyBatches"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("rollupID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    indexed: true,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("numBatch"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("stateRoot"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("exitRoot"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("aggregator"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: true,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("VerifyBatchesTrustedAggregator"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned(
                                "VerifyBatchesTrustedAggregator",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("rollupID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    indexed: true,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("numBatch"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("stateRoot"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("exitRoot"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("aggregator"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: true,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
            ]),
            errors: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned(
                        "AccessControlOnlyCanRenounceRolesForSelf",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "AccessControlOnlyCanRenounceRolesForSelf",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("AddressDoNotHaveRequiredRole"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "AddressDoNotHaveRequiredRole",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned(
                        "AllzkEVMSequencedBatchesMustBeVerified",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "AllzkEVMSequencedBatchesMustBeVerified",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("BatchFeeOutOfRange"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("BatchFeeOutOfRange"),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("ChainIDAlreadyExist"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "ChainIDAlreadyExist",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("ExceedMaxVerifyBatches"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "ExceedMaxVerifyBatches",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned(
                        "FinalNumBatchBelowLastVerifiedBatch",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "FinalNumBatchBelowLastVerifiedBatch",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned(
                        "FinalNumBatchDoesNotMatchPendingState",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "FinalNumBatchDoesNotMatchPendingState",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("FinalPendingStateNumInvalid"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "FinalPendingStateNumInvalid",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("HaltTimeoutNotExpired"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "HaltTimeoutNotExpired",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("InitBatchMustMatchCurrentForkID"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "InitBatchMustMatchCurrentForkID",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned(
                        "InitNumBatchAboveLastVerifiedBatch",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "InitNumBatchAboveLastVerifiedBatch",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned(
                        "InitNumBatchDoesNotMatchPendingState",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "InitNumBatchDoesNotMatchPendingState",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("InvalidProof"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("InvalidProof"),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("InvalidRangeBatchTimeTarget"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "InvalidRangeBatchTimeTarget",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("InvalidRangeMultiplierBatchFee"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "InvalidRangeMultiplierBatchFee",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("MustSequenceSomeBatch"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "MustSequenceSomeBatch",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("NewAccInputHashDoesNotExist"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "NewAccInputHashDoesNotExist",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned(
                        "NewPendingStateTimeoutMustBeLower",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "NewPendingStateTimeoutMustBeLower",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("NewStateRootNotInsidePrime"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "NewStateRootNotInsidePrime",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned(
                        "NewTrustedAggregatorTimeoutMustBeLower",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "NewTrustedAggregatorTimeoutMustBeLower",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("OldAccInputHashDoesNotExist"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "OldAccInputHashDoesNotExist",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("OldStateRootDoesNotExist"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "OldStateRootDoesNotExist",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("OnlyEmergencyState"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("OnlyEmergencyState"),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("OnlyNotEmergencyState"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "OnlyNotEmergencyState",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("PendingStateDoesNotExist"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "PendingStateDoesNotExist",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("PendingStateInvalid"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "PendingStateInvalid",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("PendingStateNotConsolidable"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "PendingStateNotConsolidable",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("RollupAddressAlreadyExist"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "RollupAddressAlreadyExist",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("RollupMustExist"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("RollupMustExist"),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("RollupTypeDoesNotExist"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "RollupTypeDoesNotExist",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("RollupTypeObsolete"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("RollupTypeObsolete"),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("SenderMustBeRollup"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("SenderMustBeRollup"),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned(
                        "StoredRootMustBeDifferentThanNewRoot",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "StoredRootMustBeDifferentThanNewRoot",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned(
                        "TrustedAggregatorTimeoutNotExpired",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "TrustedAggregatorTimeoutNotExpired",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("UpdateNotCompatible"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "UpdateNotCompatible",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("UpdateToSameRollupTypeID"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "UpdateToSameRollupTypeID",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
            ]),
            receive: false,
            fallback: false,
        }
    }
    ///The parsed JSON ABI of the contract.
    pub static POLYGONROLLUPMANAGER_ABI: ::ethers::contract::Lazy<
        ::ethers::core::abi::Abi,
    > = ::ethers::contract::Lazy::new(__abi);
    pub struct PolygonRollupManager<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for PolygonRollupManager<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for PolygonRollupManager<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for PolygonRollupManager<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for PolygonRollupManager<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(::core::stringify!(PolygonRollupManager))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> PolygonRollupManager<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(
                ::ethers::contract::Contract::new(
                    address.into(),
                    POLYGONROLLUPMANAGER_ABI.clone(),
                    client,
                ),
            )
        }
        ///Calls the contract's `DEFAULT_ADMIN_ROLE` (0xa217fddf) function
        pub fn default_admin_role(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([162, 23, 253, 223], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `activateEmergencyState` (0x2072f6c5) function
        pub fn activate_emergency_state(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([32, 114, 246, 197], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `addExistingRollup` (0xe0bfd3d2) function
        pub fn add_existing_rollup(
            &self,
            rollup_address: ::ethers::core::types::Address,
            verifier: ::ethers::core::types::Address,
            fork_id: u64,
            chain_id: u64,
            genesis: [u8; 32],
            rollup_compatibility_id: u8,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [224, 191, 211, 210],
                    (
                        rollup_address,
                        verifier,
                        fork_id,
                        chain_id,
                        genesis,
                        rollup_compatibility_id,
                    ),
                )
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `addNewRollupType` (0xf34eb8eb) function
        pub fn add_new_rollup_type(
            &self,
            consensus_implementation: ::ethers::core::types::Address,
            verifier: ::ethers::core::types::Address,
            fork_id: u64,
            rollup_compatibility_id: u8,
            genesis: [u8; 32],
            description: ::std::string::String,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [243, 78, 184, 235],
                    (
                        consensus_implementation,
                        verifier,
                        fork_id,
                        rollup_compatibility_id,
                        genesis,
                        description,
                    ),
                )
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `bridgeAddress` (0xa3c573eb) function
        pub fn bridge_address(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Address,
        > {
            self.0
                .method_hash([163, 197, 115, 235], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `calculateRewardPerBatch` (0x99f5634e) function
        pub fn calculate_reward_per_batch(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([153, 245, 99, 78], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `chainIDToRollupID` (0x7fb6e76a) function
        pub fn chain_id_to_rollup_id(
            &self,
            chain_id: u64,
        ) -> ::ethers::contract::builders::ContractCall<M, u32> {
            self.0
                .method_hash([127, 182, 231, 106], chain_id)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `consolidatePendingState` (0x1608859c) function
        pub fn consolidate_pending_state(
            &self,
            rollup_id: u32,
            pending_state_num: u64,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([22, 8, 133, 156], (rollup_id, pending_state_num))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `createNewRollup` (0x727885e9) function
        pub fn create_new_rollup(
            &self,
            rollup_type_id: u32,
            chain_id: u64,
            admin: ::ethers::core::types::Address,
            sequencer: ::ethers::core::types::Address,
            gas_token_address: ::ethers::core::types::Address,
            sequencer_url: ::std::string::String,
            network_name: ::std::string::String,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [114, 120, 133, 233],
                    (
                        rollup_type_id,
                        chain_id,
                        admin,
                        sequencer,
                        gas_token_address,
                        sequencer_url,
                        network_name,
                    ),
                )
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `deactivateEmergencyState` (0xdbc16976) function
        pub fn deactivate_emergency_state(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([219, 193, 105, 118], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getBatchFee` (0x477fa270) function
        pub fn get_batch_fee(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([71, 127, 162, 112], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getForcedBatchFee` (0x60469169) function
        pub fn get_forced_batch_fee(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([96, 70, 145, 105], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getInputSnarkBytes` (0x7975fcfe) function
        pub fn get_input_snark_bytes(
            &self,
            rollup_id: u32,
            init_num_batch: u64,
            final_new_batch: u64,
            new_local_exit_root: [u8; 32],
            old_state_root: [u8; 32],
            new_state_root: [u8; 32],
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Bytes,
        > {
            self.0
                .method_hash(
                    [121, 117, 252, 254],
                    (
                        rollup_id,
                        init_num_batch,
                        final_new_batch,
                        new_local_exit_root,
                        old_state_root,
                        new_state_root,
                    ),
                )
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getLastVerifiedBatch` (0x11f6b287) function
        pub fn get_last_verified_batch(
            &self,
            rollup_id: u32,
        ) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([17, 246, 178, 135], rollup_id)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getRoleAdmin` (0x248a9ca3) function
        pub fn get_role_admin(
            &self,
            role: [u8; 32],
        ) -> ::ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([36, 138, 156, 163], role)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getRollupBatchNumToStateRoot` (0x55a71ee0) function
        pub fn get_rollup_batch_num_to_state_root(
            &self,
            rollup_id: u32,
            batch_num: u64,
        ) -> ::ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([85, 167, 30, 224], (rollup_id, batch_num))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getRollupExitRoot` (0xa2967d99) function
        pub fn get_rollup_exit_root(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([162, 150, 125, 153], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getRollupPendingStateTransitions` (0xb99d0ad7) function
        pub fn get_rollup_pending_state_transitions(
            &self,
            rollup_id: u32,
            batch_num: u64,
        ) -> ::ethers::contract::builders::ContractCall<M, PendingState> {
            self.0
                .method_hash([185, 157, 10, 215], (rollup_id, batch_num))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getRollupSequencedBatches` (0x25280169) function
        pub fn get_rollup_sequenced_batches(
            &self,
            rollup_id: u32,
            batch_num: u64,
        ) -> ::ethers::contract::builders::ContractCall<M, SequencedBatchData> {
            self.0
                .method_hash([37, 40, 1, 105], (rollup_id, batch_num))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `globalExitRootManager` (0xd02103ca) function
        pub fn global_exit_root_manager(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Address,
        > {
            self.0
                .method_hash([208, 33, 3, 202], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `grantRole` (0x2f2ff15d) function
        pub fn grant_role(
            &self,
            role: [u8; 32],
            account: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([47, 47, 241, 93], (role, account))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `hasRole` (0x91d14854) function
        pub fn has_role(
            &self,
            role: [u8; 32],
            account: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([145, 209, 72, 84], (role, account))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `initialize` (0x0645af09) function
        pub fn initialize(
            &self,
            trusted_aggregator: ::ethers::core::types::Address,
            pending_state_timeout: u64,
            trusted_aggregator_timeout: u64,
            admin: ::ethers::core::types::Address,
            timelock: ::ethers::core::types::Address,
            emergency_council: ::ethers::core::types::Address,
            polygon_zk_evm: ::ethers::core::types::Address,
            zk_evm_verifier: ::ethers::core::types::Address,
            zk_evm_fork_id: u64,
            zk_evm_chain_id: u64,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [6, 69, 175, 9],
                    (
                        trusted_aggregator,
                        pending_state_timeout,
                        trusted_aggregator_timeout,
                        admin,
                        timelock,
                        emergency_council,
                        polygon_zk_evm,
                        zk_evm_verifier,
                        zk_evm_fork_id,
                        zk_evm_chain_id,
                    ),
                )
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `isEmergencyState` (0x15064c96) function
        pub fn is_emergency_state(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([21, 6, 76, 150], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `isPendingStateConsolidable` (0x080b3111) function
        pub fn is_pending_state_consolidable(
            &self,
            rollup_id: u32,
            pending_state_num: u64,
        ) -> ::ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([8, 11, 49, 17], (rollup_id, pending_state_num))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `lastAggregationTimestamp` (0xc1acbc34) function
        pub fn last_aggregation_timestamp(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([193, 172, 188, 52], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `lastDeactivatedEmergencyStateTimestamp` (0x30c27dde) function
        pub fn last_deactivated_emergency_state_timestamp(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([48, 194, 125, 222], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `multiplierBatchFee` (0xafd23cbe) function
        pub fn multiplier_batch_fee(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, u16> {
            self.0
                .method_hash([175, 210, 60, 190], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `obsoleteRollupType` (0x7222020f) function
        pub fn obsolete_rollup_type(
            &self,
            rollup_type_id: u32,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([114, 34, 2, 15], rollup_type_id)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `onSequenceBatches` (0x9a908e73) function
        pub fn on_sequence_batches(
            &self,
            new_sequenced_batches: u64,
            new_acc_input_hash: [u8; 32],
        ) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash(
                    [154, 144, 142, 115],
                    (new_sequenced_batches, new_acc_input_hash),
                )
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `overridePendingState` (0x12b86e19) function
        pub fn override_pending_state(
            &self,
            rollup_id: u32,
            init_pending_state_num: u64,
            final_pending_state_num: u64,
            init_num_batch: u64,
            final_new_batch: u64,
            new_local_exit_root: [u8; 32],
            new_state_root: [u8; 32],
            proof: [[u8; 32]; 24],
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [18, 184, 110, 25],
                    (
                        rollup_id,
                        init_pending_state_num,
                        final_pending_state_num,
                        init_num_batch,
                        final_new_batch,
                        new_local_exit_root,
                        new_state_root,
                        proof,
                    ),
                )
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `pendingStateTimeout` (0xd939b315) function
        pub fn pending_state_timeout(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([217, 57, 179, 21], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `pol` (0xe46761c4) function
        pub fn pol(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Address,
        > {
            self.0
                .method_hash([228, 103, 97, 196], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `proveNonDeterministicPendingState` (0x8bd4f071) function
        pub fn prove_non_deterministic_pending_state(
            &self,
            rollup_id: u32,
            init_pending_state_num: u64,
            final_pending_state_num: u64,
            init_num_batch: u64,
            final_new_batch: u64,
            new_local_exit_root: [u8; 32],
            new_state_root: [u8; 32],
            proof: [[u8; 32]; 24],
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [139, 212, 240, 113],
                    (
                        rollup_id,
                        init_pending_state_num,
                        final_pending_state_num,
                        init_num_batch,
                        final_new_batch,
                        new_local_exit_root,
                        new_state_root,
                        proof,
                    ),
                )
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `renounceRole` (0x36568abe) function
        pub fn renounce_role(
            &self,
            role: [u8; 32],
            account: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([54, 86, 138, 190], (role, account))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `revokeRole` (0xd547741f) function
        pub fn revoke_role(
            &self,
            role: [u8; 32],
            account: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([213, 71, 116, 31], (role, account))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `rollupAddressToID` (0xceee281d) function
        pub fn rollup_address_to_id(
            &self,
            rollup_address: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, u32> {
            self.0
                .method_hash([206, 238, 40, 29], rollup_address)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `rollupCount` (0xf4e92675) function
        pub fn rollup_count(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, u32> {
            self.0
                .method_hash([244, 233, 38, 117], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `rollupIDToRollupData` (0xf9c4c2ae) function
        pub fn rollup_id_to_rollup_data(
            &self,
            rollup_id: u32,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            (
                ::ethers::core::types::Address,
                u64,
                ::ethers::core::types::Address,
                u64,
                [u8; 32],
                u64,
                u64,
                u64,
                u64,
                u64,
                u64,
                u8,
            ),
        > {
            self.0
                .method_hash([249, 196, 194, 174], rollup_id)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `rollupTypeCount` (0x1796a1ae) function
        pub fn rollup_type_count(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, u32> {
            self.0
                .method_hash([23, 150, 161, 174], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `rollupTypeMap` (0x65c0504d) function
        pub fn rollup_type_map(
            &self,
            rollup_type_id: u32,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            (
                ::ethers::core::types::Address,
                ::ethers::core::types::Address,
                u64,
                u8,
                bool,
                [u8; 32],
            ),
        > {
            self.0
                .method_hash([101, 192, 80, 77], rollup_type_id)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `setBatchFee` (0xd5073f6f) function
        pub fn set_batch_fee(
            &self,
            new_batch_fee: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([213, 7, 63, 111], new_batch_fee)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `setMultiplierBatchFee` (0x1816b7e5) function
        pub fn set_multiplier_batch_fee(
            &self,
            new_multiplier_batch_fee: u16,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([24, 22, 183, 229], new_multiplier_batch_fee)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `setPendingStateTimeout` (0x9c9f3dfe) function
        pub fn set_pending_state_timeout(
            &self,
            new_pending_state_timeout: u64,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([156, 159, 61, 254], new_pending_state_timeout)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `setTrustedAggregatorTimeout` (0x394218e9) function
        pub fn set_trusted_aggregator_timeout(
            &self,
            new_trusted_aggregator_timeout: u64,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([57, 66, 24, 233], new_trusted_aggregator_timeout)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `setVerifyBatchTimeTarget` (0xa066215c) function
        pub fn set_verify_batch_time_target(
            &self,
            new_verify_batch_time_target: u64,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([160, 102, 33, 92], new_verify_batch_time_target)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `totalSequencedBatches` (0x066ec012) function
        pub fn total_sequenced_batches(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([6, 110, 192, 18], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `totalVerifiedBatches` (0xdde0ff77) function
        pub fn total_verified_batches(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([221, 224, 255, 119], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `trustedAggregatorTimeout` (0x841b24d7) function
        pub fn trusted_aggregator_timeout(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([132, 27, 36, 215], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `updateRollup` (0xc4c928c2) function
        pub fn update_rollup(
            &self,
            rollup_contract: ::ethers::core::types::Address,
            new_rollup_type_id: u32,
            upgrade_data: ::ethers::core::types::Bytes,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [196, 201, 40, 194],
                    (rollup_contract, new_rollup_type_id, upgrade_data),
                )
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `verifyBatchTimeTarget` (0x0a0d9fbe) function
        pub fn verify_batch_time_target(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([10, 13, 159, 190], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `verifyBatches` (0x87c20c01) function
        pub fn verify_batches(
            &self,
            rollup_id: u32,
            pending_state_num: u64,
            init_num_batch: u64,
            final_new_batch: u64,
            new_local_exit_root: [u8; 32],
            new_state_root: [u8; 32],
            beneficiary: ::ethers::core::types::Address,
            proof: [[u8; 32]; 24],
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [135, 194, 12, 1],
                    (
                        rollup_id,
                        pending_state_num,
                        init_num_batch,
                        final_new_batch,
                        new_local_exit_root,
                        new_state_root,
                        beneficiary,
                        proof,
                    ),
                )
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `verifyBatchesTrustedAggregator` (0x1489ed10) function
        pub fn verify_batches_trusted_aggregator(
            &self,
            rollup_id: u32,
            pending_state_num: u64,
            init_num_batch: u64,
            final_new_batch: u64,
            new_local_exit_root: [u8; 32],
            new_state_root: [u8; 32],
            beneficiary: ::ethers::core::types::Address,
            proof: [[u8; 32]; 24],
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [20, 137, 237, 16],
                    (
                        rollup_id,
                        pending_state_num,
                        init_num_batch,
                        final_new_batch,
                        new_local_exit_root,
                        new_state_root,
                        beneficiary,
                        proof,
                    ),
                )
                .expect("method not found (this should never happen)")
        }
        ///Gets the contract's `AddExistingRollup` event
        pub fn add_existing_rollup_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            AddExistingRollupFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `AddNewRollupType` event
        pub fn add_new_rollup_type_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            AddNewRollupTypeFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `ConsolidatePendingState` event
        pub fn consolidate_pending_state_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            ConsolidatePendingStateFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `CreateNewRollup` event
        pub fn create_new_rollup_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            CreateNewRollupFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `EmergencyStateActivated` event
        pub fn emergency_state_activated_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            EmergencyStateActivatedFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `EmergencyStateDeactivated` event
        pub fn emergency_state_deactivated_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            EmergencyStateDeactivatedFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `Initialized` event
        pub fn initialized_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            InitializedFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `ObsoleteRollupType` event
        pub fn obsolete_rollup_type_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            ObsoleteRollupTypeFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `OnSequenceBatches` event
        pub fn on_sequence_batches_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            OnSequenceBatchesFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `OverridePendingState` event
        pub fn override_pending_state_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            OverridePendingStateFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `ProveNonDeterministicPendingState` event
        pub fn prove_non_deterministic_pending_state_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            ProveNonDeterministicPendingStateFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `RoleAdminChanged` event
        pub fn role_admin_changed_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            RoleAdminChangedFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `RoleGranted` event
        pub fn role_granted_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            RoleGrantedFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `RoleRevoked` event
        pub fn role_revoked_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            RoleRevokedFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `SetBatchFee` event
        pub fn set_batch_fee_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            SetBatchFeeFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `SetMultiplierBatchFee` event
        pub fn set_multiplier_batch_fee_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            SetMultiplierBatchFeeFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `SetPendingStateTimeout` event
        pub fn set_pending_state_timeout_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            SetPendingStateTimeoutFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `SetTrustedAggregator` event
        pub fn set_trusted_aggregator_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            SetTrustedAggregatorFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `SetTrustedAggregatorTimeout` event
        pub fn set_trusted_aggregator_timeout_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            SetTrustedAggregatorTimeoutFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `SetVerifyBatchTimeTarget` event
        pub fn set_verify_batch_time_target_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            SetVerifyBatchTimeTargetFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `UpdateRollup` event
        pub fn update_rollup_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            UpdateRollupFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `VerifyBatches` event
        pub fn verify_batches_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            VerifyBatchesFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `VerifyBatchesTrustedAggregator` event
        pub fn verify_batches_trusted_aggregator_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            VerifyBatchesTrustedAggregatorFilter,
        > {
            self.0.event()
        }
        /// Returns an `Event` builder for all the events of this contract.
        pub fn events(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            PolygonRollupManagerEvents,
        > {
            self.0.event_with_filter(::core::default::Default::default())
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
    for PolygonRollupManager<M> {
        fn from(contract: ::ethers::contract::Contract<M>) -> Self {
            Self::new(contract.address(), contract.client())
        }
    }
    ///Custom Error type `AccessControlOnlyCanRenounceRolesForSelf` with signature `AccessControlOnlyCanRenounceRolesForSelf()` and selector `0x5a568e68`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(
        name = "AccessControlOnlyCanRenounceRolesForSelf",
        abi = "AccessControlOnlyCanRenounceRolesForSelf()"
    )]
    pub struct AccessControlOnlyCanRenounceRolesForSelf;
    ///Custom Error type `AddressDoNotHaveRequiredRole` with signature `AddressDoNotHaveRequiredRole()` and selector `0xec2b7c3e`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(
        name = "AddressDoNotHaveRequiredRole",
        abi = "AddressDoNotHaveRequiredRole()"
    )]
    pub struct AddressDoNotHaveRequiredRole;
    ///Custom Error type `AllzkEVMSequencedBatchesMustBeVerified` with signature `AllzkEVMSequencedBatchesMustBeVerified()` and selector `0x5c998a86`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(
        name = "AllzkEVMSequencedBatchesMustBeVerified",
        abi = "AllzkEVMSequencedBatchesMustBeVerified()"
    )]
    pub struct AllzkEVMSequencedBatchesMustBeVerified;
    ///Custom Error type `BatchFeeOutOfRange` with signature `BatchFeeOutOfRange()` and selector `0x85869525`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "BatchFeeOutOfRange", abi = "BatchFeeOutOfRange()")]
    pub struct BatchFeeOutOfRange;
    ///Custom Error type `ChainIDAlreadyExist` with signature `ChainIDAlreadyExist()` and selector `0x6f91fc12`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "ChainIDAlreadyExist", abi = "ChainIDAlreadyExist()")]
    pub struct ChainIDAlreadyExist;
    ///Custom Error type `ExceedMaxVerifyBatches` with signature `ExceedMaxVerifyBatches()` and selector `0xb59f753a`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "ExceedMaxVerifyBatches", abi = "ExceedMaxVerifyBatches()")]
    pub struct ExceedMaxVerifyBatches;
    ///Custom Error type `FinalNumBatchBelowLastVerifiedBatch` with signature `FinalNumBatchBelowLastVerifiedBatch()` and selector `0xb9b18f57`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(
        name = "FinalNumBatchBelowLastVerifiedBatch",
        abi = "FinalNumBatchBelowLastVerifiedBatch()"
    )]
    pub struct FinalNumBatchBelowLastVerifiedBatch;
    ///Custom Error type `FinalNumBatchDoesNotMatchPendingState` with signature `FinalNumBatchDoesNotMatchPendingState()` and selector `0x32a2a77f`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(
        name = "FinalNumBatchDoesNotMatchPendingState",
        abi = "FinalNumBatchDoesNotMatchPendingState()"
    )]
    pub struct FinalNumBatchDoesNotMatchPendingState;
    ///Custom Error type `FinalPendingStateNumInvalid` with signature `FinalPendingStateNumInvalid()` and selector `0xbfa7079f`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(
        name = "FinalPendingStateNumInvalid",
        abi = "FinalPendingStateNumInvalid()"
    )]
    pub struct FinalPendingStateNumInvalid;
    ///Custom Error type `HaltTimeoutNotExpired` with signature `HaltTimeoutNotExpired()` and selector `0xd257555a`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "HaltTimeoutNotExpired", abi = "HaltTimeoutNotExpired()")]
    pub struct HaltTimeoutNotExpired;
    ///Custom Error type `InitBatchMustMatchCurrentForkID` with signature `InitBatchMustMatchCurrentForkID()` and selector `0xead1340b`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(
        name = "InitBatchMustMatchCurrentForkID",
        abi = "InitBatchMustMatchCurrentForkID()"
    )]
    pub struct InitBatchMustMatchCurrentForkID;
    ///Custom Error type `InitNumBatchAboveLastVerifiedBatch` with signature `InitNumBatchAboveLastVerifiedBatch()` and selector `0x1e56e9e2`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(
        name = "InitNumBatchAboveLastVerifiedBatch",
        abi = "InitNumBatchAboveLastVerifiedBatch()"
    )]
    pub struct InitNumBatchAboveLastVerifiedBatch;
    ///Custom Error type `InitNumBatchDoesNotMatchPendingState` with signature `InitNumBatchDoesNotMatchPendingState()` and selector `0x2bd2e3e7`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(
        name = "InitNumBatchDoesNotMatchPendingState",
        abi = "InitNumBatchDoesNotMatchPendingState()"
    )]
    pub struct InitNumBatchDoesNotMatchPendingState;
    ///Custom Error type `InvalidProof` with signature `InvalidProof()` and selector `0x09bde339`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "InvalidProof", abi = "InvalidProof()")]
    pub struct InvalidProof;
    ///Custom Error type `InvalidRangeBatchTimeTarget` with signature `InvalidRangeBatchTimeTarget()` and selector `0xe067dfe8`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(
        name = "InvalidRangeBatchTimeTarget",
        abi = "InvalidRangeBatchTimeTarget()"
    )]
    pub struct InvalidRangeBatchTimeTarget;
    ///Custom Error type `InvalidRangeMultiplierBatchFee` with signature `InvalidRangeMultiplierBatchFee()` and selector `0x4c2533c8`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(
        name = "InvalidRangeMultiplierBatchFee",
        abi = "InvalidRangeMultiplierBatchFee()"
    )]
    pub struct InvalidRangeMultiplierBatchFee;
    ///Custom Error type `MustSequenceSomeBatch` with signature `MustSequenceSomeBatch()` and selector `0x2590ccf9`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "MustSequenceSomeBatch", abi = "MustSequenceSomeBatch()")]
    pub struct MustSequenceSomeBatch;
    ///Custom Error type `NewAccInputHashDoesNotExist` with signature `NewAccInputHashDoesNotExist()` and selector `0x66385b51`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(
        name = "NewAccInputHashDoesNotExist",
        abi = "NewAccInputHashDoesNotExist()"
    )]
    pub struct NewAccInputHashDoesNotExist;
    ///Custom Error type `NewPendingStateTimeoutMustBeLower` with signature `NewPendingStateTimeoutMustBeLower()` and selector `0x48a05a90`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(
        name = "NewPendingStateTimeoutMustBeLower",
        abi = "NewPendingStateTimeoutMustBeLower()"
    )]
    pub struct NewPendingStateTimeoutMustBeLower;
    ///Custom Error type `NewStateRootNotInsidePrime` with signature `NewStateRootNotInsidePrime()` and selector `0x176b913c`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(
        name = "NewStateRootNotInsidePrime",
        abi = "NewStateRootNotInsidePrime()"
    )]
    pub struct NewStateRootNotInsidePrime;
    ///Custom Error type `NewTrustedAggregatorTimeoutMustBeLower` with signature `NewTrustedAggregatorTimeoutMustBeLower()` and selector `0x401636df`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(
        name = "NewTrustedAggregatorTimeoutMustBeLower",
        abi = "NewTrustedAggregatorTimeoutMustBeLower()"
    )]
    pub struct NewTrustedAggregatorTimeoutMustBeLower;
    ///Custom Error type `OldAccInputHashDoesNotExist` with signature `OldAccInputHashDoesNotExist()` and selector `0x6818c29e`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(
        name = "OldAccInputHashDoesNotExist",
        abi = "OldAccInputHashDoesNotExist()"
    )]
    pub struct OldAccInputHashDoesNotExist;
    ///Custom Error type `OldStateRootDoesNotExist` with signature `OldStateRootDoesNotExist()` and selector `0x4997b986`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "OldStateRootDoesNotExist", abi = "OldStateRootDoesNotExist()")]
    pub struct OldStateRootDoesNotExist;
    ///Custom Error type `OnlyEmergencyState` with signature `OnlyEmergencyState()` and selector `0x53866981`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "OnlyEmergencyState", abi = "OnlyEmergencyState()")]
    pub struct OnlyEmergencyState;
    ///Custom Error type `OnlyNotEmergencyState` with signature `OnlyNotEmergencyState()` and selector `0x2f0047fc`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "OnlyNotEmergencyState", abi = "OnlyNotEmergencyState()")]
    pub struct OnlyNotEmergencyState;
    ///Custom Error type `PendingStateDoesNotExist` with signature `PendingStateDoesNotExist()` and selector `0xbb14c205`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "PendingStateDoesNotExist", abi = "PendingStateDoesNotExist()")]
    pub struct PendingStateDoesNotExist;
    ///Custom Error type `PendingStateInvalid` with signature `PendingStateInvalid()` and selector `0xd086b70b`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "PendingStateInvalid", abi = "PendingStateInvalid()")]
    pub struct PendingStateInvalid;
    ///Custom Error type `PendingStateNotConsolidable` with signature `PendingStateNotConsolidable()` and selector `0x0ce9e4a2`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(
        name = "PendingStateNotConsolidable",
        abi = "PendingStateNotConsolidable()"
    )]
    pub struct PendingStateNotConsolidable;
    ///Custom Error type `RollupAddressAlreadyExist` with signature `RollupAddressAlreadyExist()` and selector `0xd409b930`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "RollupAddressAlreadyExist", abi = "RollupAddressAlreadyExist()")]
    pub struct RollupAddressAlreadyExist;
    ///Custom Error type `RollupMustExist` with signature `RollupMustExist()` and selector `0x74a086a3`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "RollupMustExist", abi = "RollupMustExist()")]
    pub struct RollupMustExist;
    ///Custom Error type `RollupTypeDoesNotExist` with signature `RollupTypeDoesNotExist()` and selector `0x7512e5cb`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "RollupTypeDoesNotExist", abi = "RollupTypeDoesNotExist()")]
    pub struct RollupTypeDoesNotExist;
    ///Custom Error type `RollupTypeObsolete` with signature `RollupTypeObsolete()` and selector `0x3b8d3d99`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "RollupTypeObsolete", abi = "RollupTypeObsolete()")]
    pub struct RollupTypeObsolete;
    ///Custom Error type `SenderMustBeRollup` with signature `SenderMustBeRollup()` and selector `0x71653c15`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "SenderMustBeRollup", abi = "SenderMustBeRollup()")]
    pub struct SenderMustBeRollup;
    ///Custom Error type `StoredRootMustBeDifferentThanNewRoot` with signature `StoredRootMustBeDifferentThanNewRoot()` and selector `0xa47276bd`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(
        name = "StoredRootMustBeDifferentThanNewRoot",
        abi = "StoredRootMustBeDifferentThanNewRoot()"
    )]
    pub struct StoredRootMustBeDifferentThanNewRoot;
    ///Custom Error type `TrustedAggregatorTimeoutNotExpired` with signature `TrustedAggregatorTimeoutNotExpired()` and selector `0x8a0704d3`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(
        name = "TrustedAggregatorTimeoutNotExpired",
        abi = "TrustedAggregatorTimeoutNotExpired()"
    )]
    pub struct TrustedAggregatorTimeoutNotExpired;
    ///Custom Error type `UpdateNotCompatible` with signature `UpdateNotCompatible()` and selector `0xb541abe2`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "UpdateNotCompatible", abi = "UpdateNotCompatible()")]
    pub struct UpdateNotCompatible;
    ///Custom Error type `UpdateToSameRollupTypeID` with signature `UpdateToSameRollupTypeID()` and selector `0x4f61d519`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "UpdateToSameRollupTypeID", abi = "UpdateToSameRollupTypeID()")]
    pub struct UpdateToSameRollupTypeID;
    ///Container type for all of the contract's custom errors
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum PolygonRollupManagerErrors {
        AccessControlOnlyCanRenounceRolesForSelf(
            AccessControlOnlyCanRenounceRolesForSelf,
        ),
        AddressDoNotHaveRequiredRole(AddressDoNotHaveRequiredRole),
        AllzkEVMSequencedBatchesMustBeVerified(AllzkEVMSequencedBatchesMustBeVerified),
        BatchFeeOutOfRange(BatchFeeOutOfRange),
        ChainIDAlreadyExist(ChainIDAlreadyExist),
        ExceedMaxVerifyBatches(ExceedMaxVerifyBatches),
        FinalNumBatchBelowLastVerifiedBatch(FinalNumBatchBelowLastVerifiedBatch),
        FinalNumBatchDoesNotMatchPendingState(FinalNumBatchDoesNotMatchPendingState),
        FinalPendingStateNumInvalid(FinalPendingStateNumInvalid),
        HaltTimeoutNotExpired(HaltTimeoutNotExpired),
        InitBatchMustMatchCurrentForkID(InitBatchMustMatchCurrentForkID),
        InitNumBatchAboveLastVerifiedBatch(InitNumBatchAboveLastVerifiedBatch),
        InitNumBatchDoesNotMatchPendingState(InitNumBatchDoesNotMatchPendingState),
        InvalidProof(InvalidProof),
        InvalidRangeBatchTimeTarget(InvalidRangeBatchTimeTarget),
        InvalidRangeMultiplierBatchFee(InvalidRangeMultiplierBatchFee),
        MustSequenceSomeBatch(MustSequenceSomeBatch),
        NewAccInputHashDoesNotExist(NewAccInputHashDoesNotExist),
        NewPendingStateTimeoutMustBeLower(NewPendingStateTimeoutMustBeLower),
        NewStateRootNotInsidePrime(NewStateRootNotInsidePrime),
        NewTrustedAggregatorTimeoutMustBeLower(NewTrustedAggregatorTimeoutMustBeLower),
        OldAccInputHashDoesNotExist(OldAccInputHashDoesNotExist),
        OldStateRootDoesNotExist(OldStateRootDoesNotExist),
        OnlyEmergencyState(OnlyEmergencyState),
        OnlyNotEmergencyState(OnlyNotEmergencyState),
        PendingStateDoesNotExist(PendingStateDoesNotExist),
        PendingStateInvalid(PendingStateInvalid),
        PendingStateNotConsolidable(PendingStateNotConsolidable),
        RollupAddressAlreadyExist(RollupAddressAlreadyExist),
        RollupMustExist(RollupMustExist),
        RollupTypeDoesNotExist(RollupTypeDoesNotExist),
        RollupTypeObsolete(RollupTypeObsolete),
        SenderMustBeRollup(SenderMustBeRollup),
        StoredRootMustBeDifferentThanNewRoot(StoredRootMustBeDifferentThanNewRoot),
        TrustedAggregatorTimeoutNotExpired(TrustedAggregatorTimeoutNotExpired),
        UpdateNotCompatible(UpdateNotCompatible),
        UpdateToSameRollupTypeID(UpdateToSameRollupTypeID),
        /// The standard solidity revert string, with selector
        /// Error(string) -- 0x08c379a0
        RevertString(::std::string::String),
    }
    impl ::ethers::core::abi::AbiDecode for PolygonRollupManagerErrors {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded) = <::std::string::String as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::RevertString(decoded));
            }
            if let Ok(decoded) = <AccessControlOnlyCanRenounceRolesForSelf as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::AccessControlOnlyCanRenounceRolesForSelf(decoded));
            }
            if let Ok(decoded) = <AddressDoNotHaveRequiredRole as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::AddressDoNotHaveRequiredRole(decoded));
            }
            if let Ok(decoded) = <AllzkEVMSequencedBatchesMustBeVerified as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::AllzkEVMSequencedBatchesMustBeVerified(decoded));
            }
            if let Ok(decoded) = <BatchFeeOutOfRange as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::BatchFeeOutOfRange(decoded));
            }
            if let Ok(decoded) = <ChainIDAlreadyExist as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::ChainIDAlreadyExist(decoded));
            }
            if let Ok(decoded) = <ExceedMaxVerifyBatches as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::ExceedMaxVerifyBatches(decoded));
            }
            if let Ok(decoded) = <FinalNumBatchBelowLastVerifiedBatch as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::FinalNumBatchBelowLastVerifiedBatch(decoded));
            }
            if let Ok(decoded) = <FinalNumBatchDoesNotMatchPendingState as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::FinalNumBatchDoesNotMatchPendingState(decoded));
            }
            if let Ok(decoded) = <FinalPendingStateNumInvalid as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::FinalPendingStateNumInvalid(decoded));
            }
            if let Ok(decoded) = <HaltTimeoutNotExpired as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::HaltTimeoutNotExpired(decoded));
            }
            if let Ok(decoded) = <InitBatchMustMatchCurrentForkID as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::InitBatchMustMatchCurrentForkID(decoded));
            }
            if let Ok(decoded) = <InitNumBatchAboveLastVerifiedBatch as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::InitNumBatchAboveLastVerifiedBatch(decoded));
            }
            if let Ok(decoded) = <InitNumBatchDoesNotMatchPendingState as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::InitNumBatchDoesNotMatchPendingState(decoded));
            }
            if let Ok(decoded) = <InvalidProof as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::InvalidProof(decoded));
            }
            if let Ok(decoded) = <InvalidRangeBatchTimeTarget as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::InvalidRangeBatchTimeTarget(decoded));
            }
            if let Ok(decoded) = <InvalidRangeMultiplierBatchFee as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::InvalidRangeMultiplierBatchFee(decoded));
            }
            if let Ok(decoded) = <MustSequenceSomeBatch as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::MustSequenceSomeBatch(decoded));
            }
            if let Ok(decoded) = <NewAccInputHashDoesNotExist as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::NewAccInputHashDoesNotExist(decoded));
            }
            if let Ok(decoded) = <NewPendingStateTimeoutMustBeLower as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::NewPendingStateTimeoutMustBeLower(decoded));
            }
            if let Ok(decoded) = <NewStateRootNotInsidePrime as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::NewStateRootNotInsidePrime(decoded));
            }
            if let Ok(decoded) = <NewTrustedAggregatorTimeoutMustBeLower as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::NewTrustedAggregatorTimeoutMustBeLower(decoded));
            }
            if let Ok(decoded) = <OldAccInputHashDoesNotExist as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::OldAccInputHashDoesNotExist(decoded));
            }
            if let Ok(decoded) = <OldStateRootDoesNotExist as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::OldStateRootDoesNotExist(decoded));
            }
            if let Ok(decoded) = <OnlyEmergencyState as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::OnlyEmergencyState(decoded));
            }
            if let Ok(decoded) = <OnlyNotEmergencyState as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::OnlyNotEmergencyState(decoded));
            }
            if let Ok(decoded) = <PendingStateDoesNotExist as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::PendingStateDoesNotExist(decoded));
            }
            if let Ok(decoded) = <PendingStateInvalid as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::PendingStateInvalid(decoded));
            }
            if let Ok(decoded) = <PendingStateNotConsolidable as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::PendingStateNotConsolidable(decoded));
            }
            if let Ok(decoded) = <RollupAddressAlreadyExist as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::RollupAddressAlreadyExist(decoded));
            }
            if let Ok(decoded) = <RollupMustExist as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::RollupMustExist(decoded));
            }
            if let Ok(decoded) = <RollupTypeDoesNotExist as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::RollupTypeDoesNotExist(decoded));
            }
            if let Ok(decoded) = <RollupTypeObsolete as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::RollupTypeObsolete(decoded));
            }
            if let Ok(decoded) = <SenderMustBeRollup as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::SenderMustBeRollup(decoded));
            }
            if let Ok(decoded) = <StoredRootMustBeDifferentThanNewRoot as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::StoredRootMustBeDifferentThanNewRoot(decoded));
            }
            if let Ok(decoded) = <TrustedAggregatorTimeoutNotExpired as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::TrustedAggregatorTimeoutNotExpired(decoded));
            }
            if let Ok(decoded) = <UpdateNotCompatible as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::UpdateNotCompatible(decoded));
            }
            if let Ok(decoded) = <UpdateToSameRollupTypeID as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::UpdateToSameRollupTypeID(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for PolygonRollupManagerErrors {
        fn encode(self) -> ::std::vec::Vec<u8> {
            match self {
                Self::AccessControlOnlyCanRenounceRolesForSelf(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::AddressDoNotHaveRequiredRole(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::AllzkEVMSequencedBatchesMustBeVerified(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::BatchFeeOutOfRange(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ChainIDAlreadyExist(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ExceedMaxVerifyBatches(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::FinalNumBatchBelowLastVerifiedBatch(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::FinalNumBatchDoesNotMatchPendingState(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::FinalPendingStateNumInvalid(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::HaltTimeoutNotExpired(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InitBatchMustMatchCurrentForkID(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InitNumBatchAboveLastVerifiedBatch(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InitNumBatchDoesNotMatchPendingState(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidProof(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidRangeBatchTimeTarget(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidRangeMultiplierBatchFee(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::MustSequenceSomeBatch(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NewAccInputHashDoesNotExist(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NewPendingStateTimeoutMustBeLower(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NewStateRootNotInsidePrime(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NewTrustedAggregatorTimeoutMustBeLower(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::OldAccInputHashDoesNotExist(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::OldStateRootDoesNotExist(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::OnlyEmergencyState(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::OnlyNotEmergencyState(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::PendingStateDoesNotExist(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::PendingStateInvalid(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::PendingStateNotConsolidable(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RollupAddressAlreadyExist(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RollupMustExist(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RollupTypeDoesNotExist(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RollupTypeObsolete(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::SenderMustBeRollup(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::StoredRootMustBeDifferentThanNewRoot(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::TrustedAggregatorTimeoutNotExpired(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::UpdateNotCompatible(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::UpdateToSameRollupTypeID(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RevertString(s) => ::ethers::core::abi::AbiEncode::encode(s),
            }
        }
    }
    impl ::ethers::contract::ContractRevert for PolygonRollupManagerErrors {
        fn valid_selector(selector: [u8; 4]) -> bool {
            match selector {
                [0x08, 0xc3, 0x79, 0xa0] => true,
                _ if selector
                    == <AccessControlOnlyCanRenounceRolesForSelf as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <AddressDoNotHaveRequiredRole as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <AllzkEVMSequencedBatchesMustBeVerified as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <BatchFeeOutOfRange as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <ChainIDAlreadyExist as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <ExceedMaxVerifyBatches as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <FinalNumBatchBelowLastVerifiedBatch as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <FinalNumBatchDoesNotMatchPendingState as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <FinalPendingStateNumInvalid as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <HaltTimeoutNotExpired as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InitBatchMustMatchCurrentForkID as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InitNumBatchAboveLastVerifiedBatch as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InitNumBatchDoesNotMatchPendingState as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InvalidProof as ::ethers::contract::EthError>::selector() => true,
                _ if selector
                    == <InvalidRangeBatchTimeTarget as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InvalidRangeMultiplierBatchFee as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <MustSequenceSomeBatch as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <NewAccInputHashDoesNotExist as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <NewPendingStateTimeoutMustBeLower as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <NewStateRootNotInsidePrime as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <NewTrustedAggregatorTimeoutMustBeLower as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <OldAccInputHashDoesNotExist as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <OldStateRootDoesNotExist as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <OnlyEmergencyState as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <OnlyNotEmergencyState as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <PendingStateDoesNotExist as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <PendingStateInvalid as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <PendingStateNotConsolidable as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <RollupAddressAlreadyExist as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <RollupMustExist as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <RollupTypeDoesNotExist as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <RollupTypeObsolete as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <SenderMustBeRollup as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <StoredRootMustBeDifferentThanNewRoot as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <TrustedAggregatorTimeoutNotExpired as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <UpdateNotCompatible as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <UpdateToSameRollupTypeID as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ => false,
            }
        }
    }
    impl ::core::fmt::Display for PolygonRollupManagerErrors {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::AccessControlOnlyCanRenounceRolesForSelf(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::AddressDoNotHaveRequiredRole(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::AllzkEVMSequencedBatchesMustBeVerified(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::BatchFeeOutOfRange(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ChainIDAlreadyExist(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ExceedMaxVerifyBatches(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::FinalNumBatchBelowLastVerifiedBatch(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::FinalNumBatchDoesNotMatchPendingState(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::FinalPendingStateNumInvalid(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::HaltTimeoutNotExpired(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InitBatchMustMatchCurrentForkID(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InitNumBatchAboveLastVerifiedBatch(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InitNumBatchDoesNotMatchPendingState(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InvalidProof(element) => ::core::fmt::Display::fmt(element, f),
                Self::InvalidRangeBatchTimeTarget(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InvalidRangeMultiplierBatchFee(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::MustSequenceSomeBatch(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::NewAccInputHashDoesNotExist(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::NewPendingStateTimeoutMustBeLower(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::NewStateRootNotInsidePrime(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::NewTrustedAggregatorTimeoutMustBeLower(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::OldAccInputHashDoesNotExist(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::OldStateRootDoesNotExist(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::OnlyEmergencyState(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::OnlyNotEmergencyState(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::PendingStateDoesNotExist(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::PendingStateInvalid(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::PendingStateNotConsolidable(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RollupAddressAlreadyExist(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RollupMustExist(element) => ::core::fmt::Display::fmt(element, f),
                Self::RollupTypeDoesNotExist(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RollupTypeObsolete(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::SenderMustBeRollup(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::StoredRootMustBeDifferentThanNewRoot(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::TrustedAggregatorTimeoutNotExpired(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::UpdateNotCompatible(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::UpdateToSameRollupTypeID(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RevertString(s) => ::core::fmt::Display::fmt(s, f),
            }
        }
    }
    impl ::core::convert::From<::std::string::String> for PolygonRollupManagerErrors {
        fn from(value: String) -> Self {
            Self::RevertString(value)
        }
    }
    impl ::core::convert::From<AccessControlOnlyCanRenounceRolesForSelf>
    for PolygonRollupManagerErrors {
        fn from(value: AccessControlOnlyCanRenounceRolesForSelf) -> Self {
            Self::AccessControlOnlyCanRenounceRolesForSelf(value)
        }
    }
    impl ::core::convert::From<AddressDoNotHaveRequiredRole>
    for PolygonRollupManagerErrors {
        fn from(value: AddressDoNotHaveRequiredRole) -> Self {
            Self::AddressDoNotHaveRequiredRole(value)
        }
    }
    impl ::core::convert::From<AllzkEVMSequencedBatchesMustBeVerified>
    for PolygonRollupManagerErrors {
        fn from(value: AllzkEVMSequencedBatchesMustBeVerified) -> Self {
            Self::AllzkEVMSequencedBatchesMustBeVerified(value)
        }
    }
    impl ::core::convert::From<BatchFeeOutOfRange> for PolygonRollupManagerErrors {
        fn from(value: BatchFeeOutOfRange) -> Self {
            Self::BatchFeeOutOfRange(value)
        }
    }
    impl ::core::convert::From<ChainIDAlreadyExist> for PolygonRollupManagerErrors {
        fn from(value: ChainIDAlreadyExist) -> Self {
            Self::ChainIDAlreadyExist(value)
        }
    }
    impl ::core::convert::From<ExceedMaxVerifyBatches> for PolygonRollupManagerErrors {
        fn from(value: ExceedMaxVerifyBatches) -> Self {
            Self::ExceedMaxVerifyBatches(value)
        }
    }
    impl ::core::convert::From<FinalNumBatchBelowLastVerifiedBatch>
    for PolygonRollupManagerErrors {
        fn from(value: FinalNumBatchBelowLastVerifiedBatch) -> Self {
            Self::FinalNumBatchBelowLastVerifiedBatch(value)
        }
    }
    impl ::core::convert::From<FinalNumBatchDoesNotMatchPendingState>
    for PolygonRollupManagerErrors {
        fn from(value: FinalNumBatchDoesNotMatchPendingState) -> Self {
            Self::FinalNumBatchDoesNotMatchPendingState(value)
        }
    }
    impl ::core::convert::From<FinalPendingStateNumInvalid>
    for PolygonRollupManagerErrors {
        fn from(value: FinalPendingStateNumInvalid) -> Self {
            Self::FinalPendingStateNumInvalid(value)
        }
    }
    impl ::core::convert::From<HaltTimeoutNotExpired> for PolygonRollupManagerErrors {
        fn from(value: HaltTimeoutNotExpired) -> Self {
            Self::HaltTimeoutNotExpired(value)
        }
    }
    impl ::core::convert::From<InitBatchMustMatchCurrentForkID>
    for PolygonRollupManagerErrors {
        fn from(value: InitBatchMustMatchCurrentForkID) -> Self {
            Self::InitBatchMustMatchCurrentForkID(value)
        }
    }
    impl ::core::convert::From<InitNumBatchAboveLastVerifiedBatch>
    for PolygonRollupManagerErrors {
        fn from(value: InitNumBatchAboveLastVerifiedBatch) -> Self {
            Self::InitNumBatchAboveLastVerifiedBatch(value)
        }
    }
    impl ::core::convert::From<InitNumBatchDoesNotMatchPendingState>
    for PolygonRollupManagerErrors {
        fn from(value: InitNumBatchDoesNotMatchPendingState) -> Self {
            Self::InitNumBatchDoesNotMatchPendingState(value)
        }
    }
    impl ::core::convert::From<InvalidProof> for PolygonRollupManagerErrors {
        fn from(value: InvalidProof) -> Self {
            Self::InvalidProof(value)
        }
    }
    impl ::core::convert::From<InvalidRangeBatchTimeTarget>
    for PolygonRollupManagerErrors {
        fn from(value: InvalidRangeBatchTimeTarget) -> Self {
            Self::InvalidRangeBatchTimeTarget(value)
        }
    }
    impl ::core::convert::From<InvalidRangeMultiplierBatchFee>
    for PolygonRollupManagerErrors {
        fn from(value: InvalidRangeMultiplierBatchFee) -> Self {
            Self::InvalidRangeMultiplierBatchFee(value)
        }
    }
    impl ::core::convert::From<MustSequenceSomeBatch> for PolygonRollupManagerErrors {
        fn from(value: MustSequenceSomeBatch) -> Self {
            Self::MustSequenceSomeBatch(value)
        }
    }
    impl ::core::convert::From<NewAccInputHashDoesNotExist>
    for PolygonRollupManagerErrors {
        fn from(value: NewAccInputHashDoesNotExist) -> Self {
            Self::NewAccInputHashDoesNotExist(value)
        }
    }
    impl ::core::convert::From<NewPendingStateTimeoutMustBeLower>
    for PolygonRollupManagerErrors {
        fn from(value: NewPendingStateTimeoutMustBeLower) -> Self {
            Self::NewPendingStateTimeoutMustBeLower(value)
        }
    }
    impl ::core::convert::From<NewStateRootNotInsidePrime>
    for PolygonRollupManagerErrors {
        fn from(value: NewStateRootNotInsidePrime) -> Self {
            Self::NewStateRootNotInsidePrime(value)
        }
    }
    impl ::core::convert::From<NewTrustedAggregatorTimeoutMustBeLower>
    for PolygonRollupManagerErrors {
        fn from(value: NewTrustedAggregatorTimeoutMustBeLower) -> Self {
            Self::NewTrustedAggregatorTimeoutMustBeLower(value)
        }
    }
    impl ::core::convert::From<OldAccInputHashDoesNotExist>
    for PolygonRollupManagerErrors {
        fn from(value: OldAccInputHashDoesNotExist) -> Self {
            Self::OldAccInputHashDoesNotExist(value)
        }
    }
    impl ::core::convert::From<OldStateRootDoesNotExist> for PolygonRollupManagerErrors {
        fn from(value: OldStateRootDoesNotExist) -> Self {
            Self::OldStateRootDoesNotExist(value)
        }
    }
    impl ::core::convert::From<OnlyEmergencyState> for PolygonRollupManagerErrors {
        fn from(value: OnlyEmergencyState) -> Self {
            Self::OnlyEmergencyState(value)
        }
    }
    impl ::core::convert::From<OnlyNotEmergencyState> for PolygonRollupManagerErrors {
        fn from(value: OnlyNotEmergencyState) -> Self {
            Self::OnlyNotEmergencyState(value)
        }
    }
    impl ::core::convert::From<PendingStateDoesNotExist> for PolygonRollupManagerErrors {
        fn from(value: PendingStateDoesNotExist) -> Self {
            Self::PendingStateDoesNotExist(value)
        }
    }
    impl ::core::convert::From<PendingStateInvalid> for PolygonRollupManagerErrors {
        fn from(value: PendingStateInvalid) -> Self {
            Self::PendingStateInvalid(value)
        }
    }
    impl ::core::convert::From<PendingStateNotConsolidable>
    for PolygonRollupManagerErrors {
        fn from(value: PendingStateNotConsolidable) -> Self {
            Self::PendingStateNotConsolidable(value)
        }
    }
    impl ::core::convert::From<RollupAddressAlreadyExist>
    for PolygonRollupManagerErrors {
        fn from(value: RollupAddressAlreadyExist) -> Self {
            Self::RollupAddressAlreadyExist(value)
        }
    }
    impl ::core::convert::From<RollupMustExist> for PolygonRollupManagerErrors {
        fn from(value: RollupMustExist) -> Self {
            Self::RollupMustExist(value)
        }
    }
    impl ::core::convert::From<RollupTypeDoesNotExist> for PolygonRollupManagerErrors {
        fn from(value: RollupTypeDoesNotExist) -> Self {
            Self::RollupTypeDoesNotExist(value)
        }
    }
    impl ::core::convert::From<RollupTypeObsolete> for PolygonRollupManagerErrors {
        fn from(value: RollupTypeObsolete) -> Self {
            Self::RollupTypeObsolete(value)
        }
    }
    impl ::core::convert::From<SenderMustBeRollup> for PolygonRollupManagerErrors {
        fn from(value: SenderMustBeRollup) -> Self {
            Self::SenderMustBeRollup(value)
        }
    }
    impl ::core::convert::From<StoredRootMustBeDifferentThanNewRoot>
    for PolygonRollupManagerErrors {
        fn from(value: StoredRootMustBeDifferentThanNewRoot) -> Self {
            Self::StoredRootMustBeDifferentThanNewRoot(value)
        }
    }
    impl ::core::convert::From<TrustedAggregatorTimeoutNotExpired>
    for PolygonRollupManagerErrors {
        fn from(value: TrustedAggregatorTimeoutNotExpired) -> Self {
            Self::TrustedAggregatorTimeoutNotExpired(value)
        }
    }
    impl ::core::convert::From<UpdateNotCompatible> for PolygonRollupManagerErrors {
        fn from(value: UpdateNotCompatible) -> Self {
            Self::UpdateNotCompatible(value)
        }
    }
    impl ::core::convert::From<UpdateToSameRollupTypeID> for PolygonRollupManagerErrors {
        fn from(value: UpdateToSameRollupTypeID) -> Self {
            Self::UpdateToSameRollupTypeID(value)
        }
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(
        name = "AddExistingRollup",
        abi = "AddExistingRollup(uint32,uint64,address,uint64,uint8,uint64)"
    )]
    pub struct AddExistingRollupFilter {
        #[ethevent(indexed)]
        pub rollup_id: u32,
        pub fork_id: u64,
        pub rollup_address: ::ethers::core::types::Address,
        pub chain_id: u64,
        pub rollup_compatibility_id: u8,
        pub last_verified_batch_before_upgrade: u64,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(
        name = "AddNewRollupType",
        abi = "AddNewRollupType(uint32,address,address,uint64,uint8,bytes32,string)"
    )]
    pub struct AddNewRollupTypeFilter {
        #[ethevent(indexed)]
        pub rollup_type_id: u32,
        pub consensus_implementation: ::ethers::core::types::Address,
        pub verifier: ::ethers::core::types::Address,
        pub fork_id: u64,
        pub rollup_compatibility_id: u8,
        pub genesis: [u8; 32],
        pub description: ::std::string::String,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(
        name = "ConsolidatePendingState",
        abi = "ConsolidatePendingState(uint32,uint64,bytes32,bytes32,uint64)"
    )]
    pub struct ConsolidatePendingStateFilter {
        #[ethevent(indexed)]
        pub rollup_id: u32,
        pub num_batch: u64,
        pub state_root: [u8; 32],
        pub exit_root: [u8; 32],
        pub pending_state_num: u64,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(
        name = "CreateNewRollup",
        abi = "CreateNewRollup(uint32,uint32,address,uint64,address)"
    )]
    pub struct CreateNewRollupFilter {
        #[ethevent(indexed)]
        pub rollup_id: u32,
        pub rollup_type_id: u32,
        pub rollup_address: ::ethers::core::types::Address,
        pub chain_id: u64,
        pub gas_token_address: ::ethers::core::types::Address,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(name = "EmergencyStateActivated", abi = "EmergencyStateActivated()")]
    pub struct EmergencyStateActivatedFilter;
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(name = "EmergencyStateDeactivated", abi = "EmergencyStateDeactivated()")]
    pub struct EmergencyStateDeactivatedFilter;
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(name = "Initialized", abi = "Initialized(uint8)")]
    pub struct InitializedFilter {
        pub version: u8,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(name = "ObsoleteRollupType", abi = "ObsoleteRollupType(uint32)")]
    pub struct ObsoleteRollupTypeFilter {
        #[ethevent(indexed)]
        pub rollup_type_id: u32,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(name = "OnSequenceBatches", abi = "OnSequenceBatches(uint32,uint64)")]
    pub struct OnSequenceBatchesFilter {
        #[ethevent(indexed)]
        pub rollup_id: u32,
        pub last_batch_sequenced: u64,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(
        name = "OverridePendingState",
        abi = "OverridePendingState(uint32,uint64,bytes32,bytes32,address)"
    )]
    pub struct OverridePendingStateFilter {
        #[ethevent(indexed)]
        pub rollup_id: u32,
        pub num_batch: u64,
        pub state_root: [u8; 32],
        pub exit_root: [u8; 32],
        pub aggregator: ::ethers::core::types::Address,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(
        name = "ProveNonDeterministicPendingState",
        abi = "ProveNonDeterministicPendingState(bytes32,bytes32)"
    )]
    pub struct ProveNonDeterministicPendingStateFilter {
        pub stored_state_root: [u8; 32],
        pub proved_state_root: [u8; 32],
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(
        name = "RoleAdminChanged",
        abi = "RoleAdminChanged(bytes32,bytes32,bytes32)"
    )]
    pub struct RoleAdminChangedFilter {
        #[ethevent(indexed)]
        pub role: [u8; 32],
        #[ethevent(indexed)]
        pub previous_admin_role: [u8; 32],
        #[ethevent(indexed)]
        pub new_admin_role: [u8; 32],
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(name = "RoleGranted", abi = "RoleGranted(bytes32,address,address)")]
    pub struct RoleGrantedFilter {
        #[ethevent(indexed)]
        pub role: [u8; 32],
        #[ethevent(indexed)]
        pub account: ::ethers::core::types::Address,
        #[ethevent(indexed)]
        pub sender: ::ethers::core::types::Address,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(name = "RoleRevoked", abi = "RoleRevoked(bytes32,address,address)")]
    pub struct RoleRevokedFilter {
        #[ethevent(indexed)]
        pub role: [u8; 32],
        #[ethevent(indexed)]
        pub account: ::ethers::core::types::Address,
        #[ethevent(indexed)]
        pub sender: ::ethers::core::types::Address,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(name = "SetBatchFee", abi = "SetBatchFee(uint256)")]
    pub struct SetBatchFeeFilter {
        pub new_batch_fee: ::ethers::core::types::U256,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(name = "SetMultiplierBatchFee", abi = "SetMultiplierBatchFee(uint16)")]
    pub struct SetMultiplierBatchFeeFilter {
        pub new_multiplier_batch_fee: u16,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(name = "SetPendingStateTimeout", abi = "SetPendingStateTimeout(uint64)")]
    pub struct SetPendingStateTimeoutFilter {
        pub new_pending_state_timeout: u64,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(name = "SetTrustedAggregator", abi = "SetTrustedAggregator(address)")]
    pub struct SetTrustedAggregatorFilter {
        pub new_trusted_aggregator: ::ethers::core::types::Address,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(
        name = "SetTrustedAggregatorTimeout",
        abi = "SetTrustedAggregatorTimeout(uint64)"
    )]
    pub struct SetTrustedAggregatorTimeoutFilter {
        pub new_trusted_aggregator_timeout: u64,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(
        name = "SetVerifyBatchTimeTarget",
        abi = "SetVerifyBatchTimeTarget(uint64)"
    )]
    pub struct SetVerifyBatchTimeTargetFilter {
        pub new_verify_batch_time_target: u64,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(name = "UpdateRollup", abi = "UpdateRollup(uint32,uint32,uint64)")]
    pub struct UpdateRollupFilter {
        #[ethevent(indexed)]
        pub rollup_id: u32,
        pub new_rollup_type_id: u32,
        pub last_verified_batch_before_upgrade: u64,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(
        name = "VerifyBatches",
        abi = "VerifyBatches(uint32,uint64,bytes32,bytes32,address)"
    )]
    pub struct VerifyBatchesFilter {
        #[ethevent(indexed)]
        pub rollup_id: u32,
        pub num_batch: u64,
        pub state_root: [u8; 32],
        pub exit_root: [u8; 32],
        #[ethevent(indexed)]
        pub aggregator: ::ethers::core::types::Address,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethevent(
        name = "VerifyBatchesTrustedAggregator",
        abi = "VerifyBatchesTrustedAggregator(uint32,uint64,bytes32,bytes32,address)"
    )]
    pub struct VerifyBatchesTrustedAggregatorFilter {
        #[ethevent(indexed)]
        pub rollup_id: u32,
        pub num_batch: u64,
        pub state_root: [u8; 32],
        pub exit_root: [u8; 32],
        #[ethevent(indexed)]
        pub aggregator: ::ethers::core::types::Address,
    }
    ///Container type for all of the contract's events
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum PolygonRollupManagerEvents {
        AddExistingRollupFilter(AddExistingRollupFilter),
        AddNewRollupTypeFilter(AddNewRollupTypeFilter),
        ConsolidatePendingStateFilter(ConsolidatePendingStateFilter),
        CreateNewRollupFilter(CreateNewRollupFilter),
        EmergencyStateActivatedFilter(EmergencyStateActivatedFilter),
        EmergencyStateDeactivatedFilter(EmergencyStateDeactivatedFilter),
        InitializedFilter(InitializedFilter),
        ObsoleteRollupTypeFilter(ObsoleteRollupTypeFilter),
        OnSequenceBatchesFilter(OnSequenceBatchesFilter),
        OverridePendingStateFilter(OverridePendingStateFilter),
        ProveNonDeterministicPendingStateFilter(ProveNonDeterministicPendingStateFilter),
        RoleAdminChangedFilter(RoleAdminChangedFilter),
        RoleGrantedFilter(RoleGrantedFilter),
        RoleRevokedFilter(RoleRevokedFilter),
        SetBatchFeeFilter(SetBatchFeeFilter),
        SetMultiplierBatchFeeFilter(SetMultiplierBatchFeeFilter),
        SetPendingStateTimeoutFilter(SetPendingStateTimeoutFilter),
        SetTrustedAggregatorFilter(SetTrustedAggregatorFilter),
        SetTrustedAggregatorTimeoutFilter(SetTrustedAggregatorTimeoutFilter),
        SetVerifyBatchTimeTargetFilter(SetVerifyBatchTimeTargetFilter),
        UpdateRollupFilter(UpdateRollupFilter),
        VerifyBatchesFilter(VerifyBatchesFilter),
        VerifyBatchesTrustedAggregatorFilter(VerifyBatchesTrustedAggregatorFilter),
    }
    impl ::ethers::contract::EthLogDecode for PolygonRollupManagerEvents {
        fn decode_log(
            log: &::ethers::core::abi::RawLog,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::Error> {
            if let Ok(decoded) = AddExistingRollupFilter::decode_log(log) {
                return Ok(PolygonRollupManagerEvents::AddExistingRollupFilter(decoded));
            }
            if let Ok(decoded) = AddNewRollupTypeFilter::decode_log(log) {
                return Ok(PolygonRollupManagerEvents::AddNewRollupTypeFilter(decoded));
            }
            if let Ok(decoded) = ConsolidatePendingStateFilter::decode_log(log) {
                return Ok(
                    PolygonRollupManagerEvents::ConsolidatePendingStateFilter(decoded),
                );
            }
            if let Ok(decoded) = CreateNewRollupFilter::decode_log(log) {
                return Ok(PolygonRollupManagerEvents::CreateNewRollupFilter(decoded));
            }
            if let Ok(decoded) = EmergencyStateActivatedFilter::decode_log(log) {
                return Ok(
                    PolygonRollupManagerEvents::EmergencyStateActivatedFilter(decoded),
                );
            }
            if let Ok(decoded) = EmergencyStateDeactivatedFilter::decode_log(log) {
                return Ok(
                    PolygonRollupManagerEvents::EmergencyStateDeactivatedFilter(decoded),
                );
            }
            if let Ok(decoded) = InitializedFilter::decode_log(log) {
                return Ok(PolygonRollupManagerEvents::InitializedFilter(decoded));
            }
            if let Ok(decoded) = ObsoleteRollupTypeFilter::decode_log(log) {
                return Ok(PolygonRollupManagerEvents::ObsoleteRollupTypeFilter(decoded));
            }
            if let Ok(decoded) = OnSequenceBatchesFilter::decode_log(log) {
                return Ok(PolygonRollupManagerEvents::OnSequenceBatchesFilter(decoded));
            }
            if let Ok(decoded) = OverridePendingStateFilter::decode_log(log) {
                return Ok(
                    PolygonRollupManagerEvents::OverridePendingStateFilter(decoded),
                );
            }
            if let Ok(decoded) = ProveNonDeterministicPendingStateFilter::decode_log(
                log,
            ) {
                return Ok(
                    PolygonRollupManagerEvents::ProveNonDeterministicPendingStateFilter(
                        decoded,
                    ),
                );
            }
            if let Ok(decoded) = RoleAdminChangedFilter::decode_log(log) {
                return Ok(PolygonRollupManagerEvents::RoleAdminChangedFilter(decoded));
            }
            if let Ok(decoded) = RoleGrantedFilter::decode_log(log) {
                return Ok(PolygonRollupManagerEvents::RoleGrantedFilter(decoded));
            }
            if let Ok(decoded) = RoleRevokedFilter::decode_log(log) {
                return Ok(PolygonRollupManagerEvents::RoleRevokedFilter(decoded));
            }
            if let Ok(decoded) = SetBatchFeeFilter::decode_log(log) {
                return Ok(PolygonRollupManagerEvents::SetBatchFeeFilter(decoded));
            }
            if let Ok(decoded) = SetMultiplierBatchFeeFilter::decode_log(log) {
                return Ok(
                    PolygonRollupManagerEvents::SetMultiplierBatchFeeFilter(decoded),
                );
            }
            if let Ok(decoded) = SetPendingStateTimeoutFilter::decode_log(log) {
                return Ok(
                    PolygonRollupManagerEvents::SetPendingStateTimeoutFilter(decoded),
                );
            }
            if let Ok(decoded) = SetTrustedAggregatorFilter::decode_log(log) {
                return Ok(
                    PolygonRollupManagerEvents::SetTrustedAggregatorFilter(decoded),
                );
            }
            if let Ok(decoded) = SetTrustedAggregatorTimeoutFilter::decode_log(log) {
                return Ok(
                    PolygonRollupManagerEvents::SetTrustedAggregatorTimeoutFilter(
                        decoded,
                    ),
                );
            }
            if let Ok(decoded) = SetVerifyBatchTimeTargetFilter::decode_log(log) {
                return Ok(
                    PolygonRollupManagerEvents::SetVerifyBatchTimeTargetFilter(decoded),
                );
            }
            if let Ok(decoded) = UpdateRollupFilter::decode_log(log) {
                return Ok(PolygonRollupManagerEvents::UpdateRollupFilter(decoded));
            }
            if let Ok(decoded) = VerifyBatchesFilter::decode_log(log) {
                return Ok(PolygonRollupManagerEvents::VerifyBatchesFilter(decoded));
            }
            if let Ok(decoded) = VerifyBatchesTrustedAggregatorFilter::decode_log(log) {
                return Ok(
                    PolygonRollupManagerEvents::VerifyBatchesTrustedAggregatorFilter(
                        decoded,
                    ),
                );
            }
            Err(::ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::core::fmt::Display for PolygonRollupManagerEvents {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::AddExistingRollupFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::AddNewRollupTypeFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ConsolidatePendingStateFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::CreateNewRollupFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::EmergencyStateActivatedFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::EmergencyStateDeactivatedFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InitializedFilter(element) => ::core::fmt::Display::fmt(element, f),
                Self::ObsoleteRollupTypeFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::OnSequenceBatchesFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::OverridePendingStateFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ProveNonDeterministicPendingStateFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RoleAdminChangedFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RoleGrantedFilter(element) => ::core::fmt::Display::fmt(element, f),
                Self::RoleRevokedFilter(element) => ::core::fmt::Display::fmt(element, f),
                Self::SetBatchFeeFilter(element) => ::core::fmt::Display::fmt(element, f),
                Self::SetMultiplierBatchFeeFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::SetPendingStateTimeoutFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::SetTrustedAggregatorFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::SetTrustedAggregatorTimeoutFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::SetVerifyBatchTimeTargetFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::UpdateRollupFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::VerifyBatchesFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::VerifyBatchesTrustedAggregatorFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
            }
        }
    }
    impl ::core::convert::From<AddExistingRollupFilter> for PolygonRollupManagerEvents {
        fn from(value: AddExistingRollupFilter) -> Self {
            Self::AddExistingRollupFilter(value)
        }
    }
    impl ::core::convert::From<AddNewRollupTypeFilter> for PolygonRollupManagerEvents {
        fn from(value: AddNewRollupTypeFilter) -> Self {
            Self::AddNewRollupTypeFilter(value)
        }
    }
    impl ::core::convert::From<ConsolidatePendingStateFilter>
    for PolygonRollupManagerEvents {
        fn from(value: ConsolidatePendingStateFilter) -> Self {
            Self::ConsolidatePendingStateFilter(value)
        }
    }
    impl ::core::convert::From<CreateNewRollupFilter> for PolygonRollupManagerEvents {
        fn from(value: CreateNewRollupFilter) -> Self {
            Self::CreateNewRollupFilter(value)
        }
    }
    impl ::core::convert::From<EmergencyStateActivatedFilter>
    for PolygonRollupManagerEvents {
        fn from(value: EmergencyStateActivatedFilter) -> Self {
            Self::EmergencyStateActivatedFilter(value)
        }
    }
    impl ::core::convert::From<EmergencyStateDeactivatedFilter>
    for PolygonRollupManagerEvents {
        fn from(value: EmergencyStateDeactivatedFilter) -> Self {
            Self::EmergencyStateDeactivatedFilter(value)
        }
    }
    impl ::core::convert::From<InitializedFilter> for PolygonRollupManagerEvents {
        fn from(value: InitializedFilter) -> Self {
            Self::InitializedFilter(value)
        }
    }
    impl ::core::convert::From<ObsoleteRollupTypeFilter> for PolygonRollupManagerEvents {
        fn from(value: ObsoleteRollupTypeFilter) -> Self {
            Self::ObsoleteRollupTypeFilter(value)
        }
    }
    impl ::core::convert::From<OnSequenceBatchesFilter> for PolygonRollupManagerEvents {
        fn from(value: OnSequenceBatchesFilter) -> Self {
            Self::OnSequenceBatchesFilter(value)
        }
    }
    impl ::core::convert::From<OverridePendingStateFilter>
    for PolygonRollupManagerEvents {
        fn from(value: OverridePendingStateFilter) -> Self {
            Self::OverridePendingStateFilter(value)
        }
    }
    impl ::core::convert::From<ProveNonDeterministicPendingStateFilter>
    for PolygonRollupManagerEvents {
        fn from(value: ProveNonDeterministicPendingStateFilter) -> Self {
            Self::ProveNonDeterministicPendingStateFilter(value)
        }
    }
    impl ::core::convert::From<RoleAdminChangedFilter> for PolygonRollupManagerEvents {
        fn from(value: RoleAdminChangedFilter) -> Self {
            Self::RoleAdminChangedFilter(value)
        }
    }
    impl ::core::convert::From<RoleGrantedFilter> for PolygonRollupManagerEvents {
        fn from(value: RoleGrantedFilter) -> Self {
            Self::RoleGrantedFilter(value)
        }
    }
    impl ::core::convert::From<RoleRevokedFilter> for PolygonRollupManagerEvents {
        fn from(value: RoleRevokedFilter) -> Self {
            Self::RoleRevokedFilter(value)
        }
    }
    impl ::core::convert::From<SetBatchFeeFilter> for PolygonRollupManagerEvents {
        fn from(value: SetBatchFeeFilter) -> Self {
            Self::SetBatchFeeFilter(value)
        }
    }
    impl ::core::convert::From<SetMultiplierBatchFeeFilter>
    for PolygonRollupManagerEvents {
        fn from(value: SetMultiplierBatchFeeFilter) -> Self {
            Self::SetMultiplierBatchFeeFilter(value)
        }
    }
    impl ::core::convert::From<SetPendingStateTimeoutFilter>
    for PolygonRollupManagerEvents {
        fn from(value: SetPendingStateTimeoutFilter) -> Self {
            Self::SetPendingStateTimeoutFilter(value)
        }
    }
    impl ::core::convert::From<SetTrustedAggregatorFilter>
    for PolygonRollupManagerEvents {
        fn from(value: SetTrustedAggregatorFilter) -> Self {
            Self::SetTrustedAggregatorFilter(value)
        }
    }
    impl ::core::convert::From<SetTrustedAggregatorTimeoutFilter>
    for PolygonRollupManagerEvents {
        fn from(value: SetTrustedAggregatorTimeoutFilter) -> Self {
            Self::SetTrustedAggregatorTimeoutFilter(value)
        }
    }
    impl ::core::convert::From<SetVerifyBatchTimeTargetFilter>
    for PolygonRollupManagerEvents {
        fn from(value: SetVerifyBatchTimeTargetFilter) -> Self {
            Self::SetVerifyBatchTimeTargetFilter(value)
        }
    }
    impl ::core::convert::From<UpdateRollupFilter> for PolygonRollupManagerEvents {
        fn from(value: UpdateRollupFilter) -> Self {
            Self::UpdateRollupFilter(value)
        }
    }
    impl ::core::convert::From<VerifyBatchesFilter> for PolygonRollupManagerEvents {
        fn from(value: VerifyBatchesFilter) -> Self {
            Self::VerifyBatchesFilter(value)
        }
    }
    impl ::core::convert::From<VerifyBatchesTrustedAggregatorFilter>
    for PolygonRollupManagerEvents {
        fn from(value: VerifyBatchesTrustedAggregatorFilter) -> Self {
            Self::VerifyBatchesTrustedAggregatorFilter(value)
        }
    }
    ///Container type for all input parameters for the `DEFAULT_ADMIN_ROLE` function with signature `DEFAULT_ADMIN_ROLE()` and selector `0xa217fddf`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "DEFAULT_ADMIN_ROLE", abi = "DEFAULT_ADMIN_ROLE()")]
    pub struct DefaultAdminRoleCall;
    ///Container type for all input parameters for the `activateEmergencyState` function with signature `activateEmergencyState()` and selector `0x2072f6c5`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "activateEmergencyState", abi = "activateEmergencyState()")]
    pub struct ActivateEmergencyStateCall;
    ///Container type for all input parameters for the `addExistingRollup` function with signature `addExistingRollup(address,address,uint64,uint64,bytes32,uint8)` and selector `0xe0bfd3d2`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "addExistingRollup",
        abi = "addExistingRollup(address,address,uint64,uint64,bytes32,uint8)"
    )]
    pub struct AddExistingRollupCall {
        pub rollup_address: ::ethers::core::types::Address,
        pub verifier: ::ethers::core::types::Address,
        pub fork_id: u64,
        pub chain_id: u64,
        pub genesis: [u8; 32],
        pub rollup_compatibility_id: u8,
    }
    ///Container type for all input parameters for the `addNewRollupType` function with signature `addNewRollupType(address,address,uint64,uint8,bytes32,string)` and selector `0xf34eb8eb`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "addNewRollupType",
        abi = "addNewRollupType(address,address,uint64,uint8,bytes32,string)"
    )]
    pub struct AddNewRollupTypeCall {
        pub consensus_implementation: ::ethers::core::types::Address,
        pub verifier: ::ethers::core::types::Address,
        pub fork_id: u64,
        pub rollup_compatibility_id: u8,
        pub genesis: [u8; 32],
        pub description: ::std::string::String,
    }
    ///Container type for all input parameters for the `bridgeAddress` function with signature `bridgeAddress()` and selector `0xa3c573eb`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "bridgeAddress", abi = "bridgeAddress()")]
    pub struct BridgeAddressCall;
    ///Container type for all input parameters for the `calculateRewardPerBatch` function with signature `calculateRewardPerBatch()` and selector `0x99f5634e`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "calculateRewardPerBatch", abi = "calculateRewardPerBatch()")]
    pub struct CalculateRewardPerBatchCall;
    ///Container type for all input parameters for the `chainIDToRollupID` function with signature `chainIDToRollupID(uint64)` and selector `0x7fb6e76a`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "chainIDToRollupID", abi = "chainIDToRollupID(uint64)")]
    pub struct ChainIDToRollupIDCall {
        pub chain_id: u64,
    }
    ///Container type for all input parameters for the `consolidatePendingState` function with signature `consolidatePendingState(uint32,uint64)` and selector `0x1608859c`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "consolidatePendingState",
        abi = "consolidatePendingState(uint32,uint64)"
    )]
    pub struct ConsolidatePendingStateCall {
        pub rollup_id: u32,
        pub pending_state_num: u64,
    }
    ///Container type for all input parameters for the `createNewRollup` function with signature `createNewRollup(uint32,uint64,address,address,address,string,string)` and selector `0x727885e9`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "createNewRollup",
        abi = "createNewRollup(uint32,uint64,address,address,address,string,string)"
    )]
    pub struct CreateNewRollupCall {
        pub rollup_type_id: u32,
        pub chain_id: u64,
        pub admin: ::ethers::core::types::Address,
        pub sequencer: ::ethers::core::types::Address,
        pub gas_token_address: ::ethers::core::types::Address,
        pub sequencer_url: ::std::string::String,
        pub network_name: ::std::string::String,
    }
    ///Container type for all input parameters for the `deactivateEmergencyState` function with signature `deactivateEmergencyState()` and selector `0xdbc16976`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "deactivateEmergencyState", abi = "deactivateEmergencyState()")]
    pub struct DeactivateEmergencyStateCall;
    ///Container type for all input parameters for the `getBatchFee` function with signature `getBatchFee()` and selector `0x477fa270`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "getBatchFee", abi = "getBatchFee()")]
    pub struct GetBatchFeeCall;
    ///Container type for all input parameters for the `getForcedBatchFee` function with signature `getForcedBatchFee()` and selector `0x60469169`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "getForcedBatchFee", abi = "getForcedBatchFee()")]
    pub struct GetForcedBatchFeeCall;
    ///Container type for all input parameters for the `getInputSnarkBytes` function with signature `getInputSnarkBytes(uint32,uint64,uint64,bytes32,bytes32,bytes32)` and selector `0x7975fcfe`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "getInputSnarkBytes",
        abi = "getInputSnarkBytes(uint32,uint64,uint64,bytes32,bytes32,bytes32)"
    )]
    pub struct GetInputSnarkBytesCall {
        pub rollup_id: u32,
        pub init_num_batch: u64,
        pub final_new_batch: u64,
        pub new_local_exit_root: [u8; 32],
        pub old_state_root: [u8; 32],
        pub new_state_root: [u8; 32],
    }
    ///Container type for all input parameters for the `getLastVerifiedBatch` function with signature `getLastVerifiedBatch(uint32)` and selector `0x11f6b287`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "getLastVerifiedBatch", abi = "getLastVerifiedBatch(uint32)")]
    pub struct GetLastVerifiedBatchCall {
        pub rollup_id: u32,
    }
    ///Container type for all input parameters for the `getRoleAdmin` function with signature `getRoleAdmin(bytes32)` and selector `0x248a9ca3`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "getRoleAdmin", abi = "getRoleAdmin(bytes32)")]
    pub struct GetRoleAdminCall {
        pub role: [u8; 32],
    }
    ///Container type for all input parameters for the `getRollupBatchNumToStateRoot` function with signature `getRollupBatchNumToStateRoot(uint32,uint64)` and selector `0x55a71ee0`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "getRollupBatchNumToStateRoot",
        abi = "getRollupBatchNumToStateRoot(uint32,uint64)"
    )]
    pub struct GetRollupBatchNumToStateRootCall {
        pub rollup_id: u32,
        pub batch_num: u64,
    }
    ///Container type for all input parameters for the `getRollupExitRoot` function with signature `getRollupExitRoot()` and selector `0xa2967d99`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "getRollupExitRoot", abi = "getRollupExitRoot()")]
    pub struct GetRollupExitRootCall;
    ///Container type for all input parameters for the `getRollupPendingStateTransitions` function with signature `getRollupPendingStateTransitions(uint32,uint64)` and selector `0xb99d0ad7`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "getRollupPendingStateTransitions",
        abi = "getRollupPendingStateTransitions(uint32,uint64)"
    )]
    pub struct GetRollupPendingStateTransitionsCall {
        pub rollup_id: u32,
        pub batch_num: u64,
    }
    ///Container type for all input parameters for the `getRollupSequencedBatches` function with signature `getRollupSequencedBatches(uint32,uint64)` and selector `0x25280169`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "getRollupSequencedBatches",
        abi = "getRollupSequencedBatches(uint32,uint64)"
    )]
    pub struct GetRollupSequencedBatchesCall {
        pub rollup_id: u32,
        pub batch_num: u64,
    }
    ///Container type for all input parameters for the `globalExitRootManager` function with signature `globalExitRootManager()` and selector `0xd02103ca`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "globalExitRootManager", abi = "globalExitRootManager()")]
    pub struct GlobalExitRootManagerCall;
    ///Container type for all input parameters for the `grantRole` function with signature `grantRole(bytes32,address)` and selector `0x2f2ff15d`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "grantRole", abi = "grantRole(bytes32,address)")]
    pub struct GrantRoleCall {
        pub role: [u8; 32],
        pub account: ::ethers::core::types::Address,
    }
    ///Container type for all input parameters for the `hasRole` function with signature `hasRole(bytes32,address)` and selector `0x91d14854`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "hasRole", abi = "hasRole(bytes32,address)")]
    pub struct HasRoleCall {
        pub role: [u8; 32],
        pub account: ::ethers::core::types::Address,
    }
    ///Container type for all input parameters for the `initialize` function with signature `initialize(address,uint64,uint64,address,address,address,address,address,uint64,uint64)` and selector `0x0645af09`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "initialize",
        abi = "initialize(address,uint64,uint64,address,address,address,address,address,uint64,uint64)"
    )]
    pub struct InitializeCall {
        pub trusted_aggregator: ::ethers::core::types::Address,
        pub pending_state_timeout: u64,
        pub trusted_aggregator_timeout: u64,
        pub admin: ::ethers::core::types::Address,
        pub timelock: ::ethers::core::types::Address,
        pub emergency_council: ::ethers::core::types::Address,
        pub polygon_zk_evm: ::ethers::core::types::Address,
        pub zk_evm_verifier: ::ethers::core::types::Address,
        pub zk_evm_fork_id: u64,
        pub zk_evm_chain_id: u64,
    }
    ///Container type for all input parameters for the `isEmergencyState` function with signature `isEmergencyState()` and selector `0x15064c96`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "isEmergencyState", abi = "isEmergencyState()")]
    pub struct IsEmergencyStateCall;
    ///Container type for all input parameters for the `isPendingStateConsolidable` function with signature `isPendingStateConsolidable(uint32,uint64)` and selector `0x080b3111`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "isPendingStateConsolidable",
        abi = "isPendingStateConsolidable(uint32,uint64)"
    )]
    pub struct IsPendingStateConsolidableCall {
        pub rollup_id: u32,
        pub pending_state_num: u64,
    }
    ///Container type for all input parameters for the `lastAggregationTimestamp` function with signature `lastAggregationTimestamp()` and selector `0xc1acbc34`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "lastAggregationTimestamp", abi = "lastAggregationTimestamp()")]
    pub struct LastAggregationTimestampCall;
    ///Container type for all input parameters for the `lastDeactivatedEmergencyStateTimestamp` function with signature `lastDeactivatedEmergencyStateTimestamp()` and selector `0x30c27dde`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "lastDeactivatedEmergencyStateTimestamp",
        abi = "lastDeactivatedEmergencyStateTimestamp()"
    )]
    pub struct LastDeactivatedEmergencyStateTimestampCall;
    ///Container type for all input parameters for the `multiplierBatchFee` function with signature `multiplierBatchFee()` and selector `0xafd23cbe`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "multiplierBatchFee", abi = "multiplierBatchFee()")]
    pub struct MultiplierBatchFeeCall;
    ///Container type for all input parameters for the `obsoleteRollupType` function with signature `obsoleteRollupType(uint32)` and selector `0x7222020f`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "obsoleteRollupType", abi = "obsoleteRollupType(uint32)")]
    pub struct ObsoleteRollupTypeCall {
        pub rollup_type_id: u32,
    }
    ///Container type for all input parameters for the `onSequenceBatches` function with signature `onSequenceBatches(uint64,bytes32)` and selector `0x9a908e73`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "onSequenceBatches", abi = "onSequenceBatches(uint64,bytes32)")]
    pub struct OnSequenceBatchesCall {
        pub new_sequenced_batches: u64,
        pub new_acc_input_hash: [u8; 32],
    }
    ///Container type for all input parameters for the `overridePendingState` function with signature `overridePendingState(uint32,uint64,uint64,uint64,uint64,bytes32,bytes32,bytes32[24])` and selector `0x12b86e19`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "overridePendingState",
        abi = "overridePendingState(uint32,uint64,uint64,uint64,uint64,bytes32,bytes32,bytes32[24])"
    )]
    pub struct OverridePendingStateCall {
        pub rollup_id: u32,
        pub init_pending_state_num: u64,
        pub final_pending_state_num: u64,
        pub init_num_batch: u64,
        pub final_new_batch: u64,
        pub new_local_exit_root: [u8; 32],
        pub new_state_root: [u8; 32],
        pub proof: [[u8; 32]; 24],
    }
    ///Container type for all input parameters for the `pendingStateTimeout` function with signature `pendingStateTimeout()` and selector `0xd939b315`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "pendingStateTimeout", abi = "pendingStateTimeout()")]
    pub struct PendingStateTimeoutCall;
    ///Container type for all input parameters for the `pol` function with signature `pol()` and selector `0xe46761c4`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "pol", abi = "pol()")]
    pub struct PolCall;
    ///Container type for all input parameters for the `proveNonDeterministicPendingState` function with signature `proveNonDeterministicPendingState(uint32,uint64,uint64,uint64,uint64,bytes32,bytes32,bytes32[24])` and selector `0x8bd4f071`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "proveNonDeterministicPendingState",
        abi = "proveNonDeterministicPendingState(uint32,uint64,uint64,uint64,uint64,bytes32,bytes32,bytes32[24])"
    )]
    pub struct ProveNonDeterministicPendingStateCall {
        pub rollup_id: u32,
        pub init_pending_state_num: u64,
        pub final_pending_state_num: u64,
        pub init_num_batch: u64,
        pub final_new_batch: u64,
        pub new_local_exit_root: [u8; 32],
        pub new_state_root: [u8; 32],
        pub proof: [[u8; 32]; 24],
    }
    ///Container type for all input parameters for the `renounceRole` function with signature `renounceRole(bytes32,address)` and selector `0x36568abe`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "renounceRole", abi = "renounceRole(bytes32,address)")]
    pub struct RenounceRoleCall {
        pub role: [u8; 32],
        pub account: ::ethers::core::types::Address,
    }
    ///Container type for all input parameters for the `revokeRole` function with signature `revokeRole(bytes32,address)` and selector `0xd547741f`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "revokeRole", abi = "revokeRole(bytes32,address)")]
    pub struct RevokeRoleCall {
        pub role: [u8; 32],
        pub account: ::ethers::core::types::Address,
    }
    ///Container type for all input parameters for the `rollupAddressToID` function with signature `rollupAddressToID(address)` and selector `0xceee281d`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "rollupAddressToID", abi = "rollupAddressToID(address)")]
    pub struct RollupAddressToIDCall {
        pub rollup_address: ::ethers::core::types::Address,
    }
    ///Container type for all input parameters for the `rollupCount` function with signature `rollupCount()` and selector `0xf4e92675`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "rollupCount", abi = "rollupCount()")]
    pub struct RollupCountCall;
    ///Container type for all input parameters for the `rollupIDToRollupData` function with signature `rollupIDToRollupData(uint32)` and selector `0xf9c4c2ae`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "rollupIDToRollupData", abi = "rollupIDToRollupData(uint32)")]
    pub struct RollupIDToRollupDataCall {
        pub rollup_id: u32,
    }
    ///Container type for all input parameters for the `rollupTypeCount` function with signature `rollupTypeCount()` and selector `0x1796a1ae`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "rollupTypeCount", abi = "rollupTypeCount()")]
    pub struct RollupTypeCountCall;
    ///Container type for all input parameters for the `rollupTypeMap` function with signature `rollupTypeMap(uint32)` and selector `0x65c0504d`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "rollupTypeMap", abi = "rollupTypeMap(uint32)")]
    pub struct RollupTypeMapCall {
        pub rollup_type_id: u32,
    }
    ///Container type for all input parameters for the `setBatchFee` function with signature `setBatchFee(uint256)` and selector `0xd5073f6f`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "setBatchFee", abi = "setBatchFee(uint256)")]
    pub struct SetBatchFeeCall {
        pub new_batch_fee: ::ethers::core::types::U256,
    }
    ///Container type for all input parameters for the `setMultiplierBatchFee` function with signature `setMultiplierBatchFee(uint16)` and selector `0x1816b7e5`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "setMultiplierBatchFee", abi = "setMultiplierBatchFee(uint16)")]
    pub struct SetMultiplierBatchFeeCall {
        pub new_multiplier_batch_fee: u16,
    }
    ///Container type for all input parameters for the `setPendingStateTimeout` function with signature `setPendingStateTimeout(uint64)` and selector `0x9c9f3dfe`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "setPendingStateTimeout", abi = "setPendingStateTimeout(uint64)")]
    pub struct SetPendingStateTimeoutCall {
        pub new_pending_state_timeout: u64,
    }
    ///Container type for all input parameters for the `setTrustedAggregatorTimeout` function with signature `setTrustedAggregatorTimeout(uint64)` and selector `0x394218e9`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "setTrustedAggregatorTimeout",
        abi = "setTrustedAggregatorTimeout(uint64)"
    )]
    pub struct SetTrustedAggregatorTimeoutCall {
        pub new_trusted_aggregator_timeout: u64,
    }
    ///Container type for all input parameters for the `setVerifyBatchTimeTarget` function with signature `setVerifyBatchTimeTarget(uint64)` and selector `0xa066215c`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "setVerifyBatchTimeTarget",
        abi = "setVerifyBatchTimeTarget(uint64)"
    )]
    pub struct SetVerifyBatchTimeTargetCall {
        pub new_verify_batch_time_target: u64,
    }
    ///Container type for all input parameters for the `totalSequencedBatches` function with signature `totalSequencedBatches()` and selector `0x066ec012`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "totalSequencedBatches", abi = "totalSequencedBatches()")]
    pub struct TotalSequencedBatchesCall;
    ///Container type for all input parameters for the `totalVerifiedBatches` function with signature `totalVerifiedBatches()` and selector `0xdde0ff77`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "totalVerifiedBatches", abi = "totalVerifiedBatches()")]
    pub struct TotalVerifiedBatchesCall;
    ///Container type for all input parameters for the `trustedAggregatorTimeout` function with signature `trustedAggregatorTimeout()` and selector `0x841b24d7`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "trustedAggregatorTimeout", abi = "trustedAggregatorTimeout()")]
    pub struct TrustedAggregatorTimeoutCall;
    ///Container type for all input parameters for the `updateRollup` function with signature `updateRollup(address,uint32,bytes)` and selector `0xc4c928c2`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "updateRollup", abi = "updateRollup(address,uint32,bytes)")]
    pub struct UpdateRollupCall {
        pub rollup_contract: ::ethers::core::types::Address,
        pub new_rollup_type_id: u32,
        pub upgrade_data: ::ethers::core::types::Bytes,
    }
    ///Container type for all input parameters for the `verifyBatchTimeTarget` function with signature `verifyBatchTimeTarget()` and selector `0x0a0d9fbe`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "verifyBatchTimeTarget", abi = "verifyBatchTimeTarget()")]
    pub struct VerifyBatchTimeTargetCall;
    ///Container type for all input parameters for the `verifyBatches` function with signature `verifyBatches(uint32,uint64,uint64,uint64,bytes32,bytes32,address,bytes32[24])` and selector `0x87c20c01`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "verifyBatches",
        abi = "verifyBatches(uint32,uint64,uint64,uint64,bytes32,bytes32,address,bytes32[24])"
    )]
    pub struct VerifyBatchesCall {
        pub rollup_id: u32,
        pub pending_state_num: u64,
        pub init_num_batch: u64,
        pub final_new_batch: u64,
        pub new_local_exit_root: [u8; 32],
        pub new_state_root: [u8; 32],
        pub beneficiary: ::ethers::core::types::Address,
        pub proof: [[u8; 32]; 24],
    }
    ///Container type for all input parameters for the `verifyBatchesTrustedAggregator` function with signature `verifyBatchesTrustedAggregator(uint32,uint64,uint64,uint64,bytes32,bytes32,address,bytes32[24])` and selector `0x1489ed10`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "verifyBatchesTrustedAggregator",
        abi = "verifyBatchesTrustedAggregator(uint32,uint64,uint64,uint64,bytes32,bytes32,address,bytes32[24])"
    )]
    pub struct VerifyBatchesTrustedAggregatorCall {
        pub rollup_id: u32,
        pub pending_state_num: u64,
        pub init_num_batch: u64,
        pub final_new_batch: u64,
        pub new_local_exit_root: [u8; 32],
        pub new_state_root: [u8; 32],
        pub beneficiary: ::ethers::core::types::Address,
        pub proof: [[u8; 32]; 24],
    }
    ///Container type for all of the contract's call
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum PolygonRollupManagerCalls {
        DefaultAdminRole(DefaultAdminRoleCall),
        ActivateEmergencyState(ActivateEmergencyStateCall),
        AddExistingRollup(AddExistingRollupCall),
        AddNewRollupType(AddNewRollupTypeCall),
        BridgeAddress(BridgeAddressCall),
        CalculateRewardPerBatch(CalculateRewardPerBatchCall),
        ChainIDToRollupID(ChainIDToRollupIDCall),
        ConsolidatePendingState(ConsolidatePendingStateCall),
        CreateNewRollup(CreateNewRollupCall),
        DeactivateEmergencyState(DeactivateEmergencyStateCall),
        GetBatchFee(GetBatchFeeCall),
        GetForcedBatchFee(GetForcedBatchFeeCall),
        GetInputSnarkBytes(GetInputSnarkBytesCall),
        GetLastVerifiedBatch(GetLastVerifiedBatchCall),
        GetRoleAdmin(GetRoleAdminCall),
        GetRollupBatchNumToStateRoot(GetRollupBatchNumToStateRootCall),
        GetRollupExitRoot(GetRollupExitRootCall),
        GetRollupPendingStateTransitions(GetRollupPendingStateTransitionsCall),
        GetRollupSequencedBatches(GetRollupSequencedBatchesCall),
        GlobalExitRootManager(GlobalExitRootManagerCall),
        GrantRole(GrantRoleCall),
        HasRole(HasRoleCall),
        Initialize(InitializeCall),
        IsEmergencyState(IsEmergencyStateCall),
        IsPendingStateConsolidable(IsPendingStateConsolidableCall),
        LastAggregationTimestamp(LastAggregationTimestampCall),
        LastDeactivatedEmergencyStateTimestamp(
            LastDeactivatedEmergencyStateTimestampCall,
        ),
        MultiplierBatchFee(MultiplierBatchFeeCall),
        ObsoleteRollupType(ObsoleteRollupTypeCall),
        OnSequenceBatches(OnSequenceBatchesCall),
        OverridePendingState(OverridePendingStateCall),
        PendingStateTimeout(PendingStateTimeoutCall),
        Pol(PolCall),
        ProveNonDeterministicPendingState(ProveNonDeterministicPendingStateCall),
        RenounceRole(RenounceRoleCall),
        RevokeRole(RevokeRoleCall),
        RollupAddressToID(RollupAddressToIDCall),
        RollupCount(RollupCountCall),
        RollupIDToRollupData(RollupIDToRollupDataCall),
        RollupTypeCount(RollupTypeCountCall),
        RollupTypeMap(RollupTypeMapCall),
        SetBatchFee(SetBatchFeeCall),
        SetMultiplierBatchFee(SetMultiplierBatchFeeCall),
        SetPendingStateTimeout(SetPendingStateTimeoutCall),
        SetTrustedAggregatorTimeout(SetTrustedAggregatorTimeoutCall),
        SetVerifyBatchTimeTarget(SetVerifyBatchTimeTargetCall),
        TotalSequencedBatches(TotalSequencedBatchesCall),
        TotalVerifiedBatches(TotalVerifiedBatchesCall),
        TrustedAggregatorTimeout(TrustedAggregatorTimeoutCall),
        UpdateRollup(UpdateRollupCall),
        VerifyBatchTimeTarget(VerifyBatchTimeTargetCall),
        VerifyBatches(VerifyBatchesCall),
        VerifyBatchesTrustedAggregator(VerifyBatchesTrustedAggregatorCall),
    }
    impl ::ethers::core::abi::AbiDecode for PolygonRollupManagerCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded) = <DefaultAdminRoleCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::DefaultAdminRole(decoded));
            }
            if let Ok(decoded) = <ActivateEmergencyStateCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::ActivateEmergencyState(decoded));
            }
            if let Ok(decoded) = <AddExistingRollupCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::AddExistingRollup(decoded));
            }
            if let Ok(decoded) = <AddNewRollupTypeCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::AddNewRollupType(decoded));
            }
            if let Ok(decoded) = <BridgeAddressCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::BridgeAddress(decoded));
            }
            if let Ok(decoded) = <CalculateRewardPerBatchCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::CalculateRewardPerBatch(decoded));
            }
            if let Ok(decoded) = <ChainIDToRollupIDCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::ChainIDToRollupID(decoded));
            }
            if let Ok(decoded) = <ConsolidatePendingStateCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::ConsolidatePendingState(decoded));
            }
            if let Ok(decoded) = <CreateNewRollupCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::CreateNewRollup(decoded));
            }
            if let Ok(decoded) = <DeactivateEmergencyStateCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::DeactivateEmergencyState(decoded));
            }
            if let Ok(decoded) = <GetBatchFeeCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GetBatchFee(decoded));
            }
            if let Ok(decoded) = <GetForcedBatchFeeCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GetForcedBatchFee(decoded));
            }
            if let Ok(decoded) = <GetInputSnarkBytesCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GetInputSnarkBytes(decoded));
            }
            if let Ok(decoded) = <GetLastVerifiedBatchCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GetLastVerifiedBatch(decoded));
            }
            if let Ok(decoded) = <GetRoleAdminCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GetRoleAdmin(decoded));
            }
            if let Ok(decoded) = <GetRollupBatchNumToStateRootCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GetRollupBatchNumToStateRoot(decoded));
            }
            if let Ok(decoded) = <GetRollupExitRootCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GetRollupExitRoot(decoded));
            }
            if let Ok(decoded) = <GetRollupPendingStateTransitionsCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GetRollupPendingStateTransitions(decoded));
            }
            if let Ok(decoded) = <GetRollupSequencedBatchesCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GetRollupSequencedBatches(decoded));
            }
            if let Ok(decoded) = <GlobalExitRootManagerCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GlobalExitRootManager(decoded));
            }
            if let Ok(decoded) = <GrantRoleCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GrantRole(decoded));
            }
            if let Ok(decoded) = <HasRoleCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::HasRole(decoded));
            }
            if let Ok(decoded) = <InitializeCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::Initialize(decoded));
            }
            if let Ok(decoded) = <IsEmergencyStateCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::IsEmergencyState(decoded));
            }
            if let Ok(decoded) = <IsPendingStateConsolidableCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::IsPendingStateConsolidable(decoded));
            }
            if let Ok(decoded) = <LastAggregationTimestampCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::LastAggregationTimestamp(decoded));
            }
            if let Ok(decoded) = <LastDeactivatedEmergencyStateTimestampCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::LastDeactivatedEmergencyStateTimestamp(decoded));
            }
            if let Ok(decoded) = <MultiplierBatchFeeCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::MultiplierBatchFee(decoded));
            }
            if let Ok(decoded) = <ObsoleteRollupTypeCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::ObsoleteRollupType(decoded));
            }
            if let Ok(decoded) = <OnSequenceBatchesCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::OnSequenceBatches(decoded));
            }
            if let Ok(decoded) = <OverridePendingStateCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::OverridePendingState(decoded));
            }
            if let Ok(decoded) = <PendingStateTimeoutCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::PendingStateTimeout(decoded));
            }
            if let Ok(decoded) = <PolCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::Pol(decoded));
            }
            if let Ok(decoded) = <ProveNonDeterministicPendingStateCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::ProveNonDeterministicPendingState(decoded));
            }
            if let Ok(decoded) = <RenounceRoleCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::RenounceRole(decoded));
            }
            if let Ok(decoded) = <RevokeRoleCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::RevokeRole(decoded));
            }
            if let Ok(decoded) = <RollupAddressToIDCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::RollupAddressToID(decoded));
            }
            if let Ok(decoded) = <RollupCountCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::RollupCount(decoded));
            }
            if let Ok(decoded) = <RollupIDToRollupDataCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::RollupIDToRollupData(decoded));
            }
            if let Ok(decoded) = <RollupTypeCountCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::RollupTypeCount(decoded));
            }
            if let Ok(decoded) = <RollupTypeMapCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::RollupTypeMap(decoded));
            }
            if let Ok(decoded) = <SetBatchFeeCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::SetBatchFee(decoded));
            }
            if let Ok(decoded) = <SetMultiplierBatchFeeCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::SetMultiplierBatchFee(decoded));
            }
            if let Ok(decoded) = <SetPendingStateTimeoutCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::SetPendingStateTimeout(decoded));
            }
            if let Ok(decoded) = <SetTrustedAggregatorTimeoutCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::SetTrustedAggregatorTimeout(decoded));
            }
            if let Ok(decoded) = <SetVerifyBatchTimeTargetCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::SetVerifyBatchTimeTarget(decoded));
            }
            if let Ok(decoded) = <TotalSequencedBatchesCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::TotalSequencedBatches(decoded));
            }
            if let Ok(decoded) = <TotalVerifiedBatchesCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::TotalVerifiedBatches(decoded));
            }
            if let Ok(decoded) = <TrustedAggregatorTimeoutCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::TrustedAggregatorTimeout(decoded));
            }
            if let Ok(decoded) = <UpdateRollupCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::UpdateRollup(decoded));
            }
            if let Ok(decoded) = <VerifyBatchTimeTargetCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::VerifyBatchTimeTarget(decoded));
            }
            if let Ok(decoded) = <VerifyBatchesCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::VerifyBatches(decoded));
            }
            if let Ok(decoded) = <VerifyBatchesTrustedAggregatorCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::VerifyBatchesTrustedAggregator(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for PolygonRollupManagerCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                Self::DefaultAdminRole(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ActivateEmergencyState(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::AddExistingRollup(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::AddNewRollupType(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::BridgeAddress(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CalculateRewardPerBatch(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ChainIDToRollupID(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ConsolidatePendingState(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CreateNewRollup(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::DeactivateEmergencyState(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetBatchFee(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetForcedBatchFee(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetInputSnarkBytes(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetLastVerifiedBatch(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetRoleAdmin(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetRollupBatchNumToStateRoot(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetRollupExitRoot(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetRollupPendingStateTransitions(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetRollupSequencedBatches(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GlobalExitRootManager(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GrantRole(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::HasRole(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::Initialize(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::IsEmergencyState(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::IsPendingStateConsolidable(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::LastAggregationTimestamp(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::LastDeactivatedEmergencyStateTimestamp(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::MultiplierBatchFee(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ObsoleteRollupType(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::OnSequenceBatches(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::OverridePendingState(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::PendingStateTimeout(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::Pol(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::ProveNonDeterministicPendingState(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RenounceRole(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RevokeRole(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RollupAddressToID(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RollupCount(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RollupIDToRollupData(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RollupTypeCount(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RollupTypeMap(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::SetBatchFee(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::SetMultiplierBatchFee(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::SetPendingStateTimeout(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::SetTrustedAggregatorTimeout(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::SetVerifyBatchTimeTarget(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::TotalSequencedBatches(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::TotalVerifiedBatches(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::TrustedAggregatorTimeout(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::UpdateRollup(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::VerifyBatchTimeTarget(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::VerifyBatches(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::VerifyBatchesTrustedAggregator(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
            }
        }
    }
    impl ::core::fmt::Display for PolygonRollupManagerCalls {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::DefaultAdminRole(element) => ::core::fmt::Display::fmt(element, f),
                Self::ActivateEmergencyState(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::AddExistingRollup(element) => ::core::fmt::Display::fmt(element, f),
                Self::AddNewRollupType(element) => ::core::fmt::Display::fmt(element, f),
                Self::BridgeAddress(element) => ::core::fmt::Display::fmt(element, f),
                Self::CalculateRewardPerBatch(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ChainIDToRollupID(element) => ::core::fmt::Display::fmt(element, f),
                Self::ConsolidatePendingState(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::CreateNewRollup(element) => ::core::fmt::Display::fmt(element, f),
                Self::DeactivateEmergencyState(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::GetBatchFee(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetForcedBatchFee(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetInputSnarkBytes(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::GetLastVerifiedBatch(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::GetRoleAdmin(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetRollupBatchNumToStateRoot(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::GetRollupExitRoot(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetRollupPendingStateTransitions(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::GetRollupSequencedBatches(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::GlobalExitRootManager(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::GrantRole(element) => ::core::fmt::Display::fmt(element, f),
                Self::HasRole(element) => ::core::fmt::Display::fmt(element, f),
                Self::Initialize(element) => ::core::fmt::Display::fmt(element, f),
                Self::IsEmergencyState(element) => ::core::fmt::Display::fmt(element, f),
                Self::IsPendingStateConsolidable(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::LastAggregationTimestamp(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::LastDeactivatedEmergencyStateTimestamp(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::MultiplierBatchFee(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ObsoleteRollupType(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::OnSequenceBatches(element) => ::core::fmt::Display::fmt(element, f),
                Self::OverridePendingState(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::PendingStateTimeout(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::Pol(element) => ::core::fmt::Display::fmt(element, f),
                Self::ProveNonDeterministicPendingState(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RenounceRole(element) => ::core::fmt::Display::fmt(element, f),
                Self::RevokeRole(element) => ::core::fmt::Display::fmt(element, f),
                Self::RollupAddressToID(element) => ::core::fmt::Display::fmt(element, f),
                Self::RollupCount(element) => ::core::fmt::Display::fmt(element, f),
                Self::RollupIDToRollupData(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RollupTypeCount(element) => ::core::fmt::Display::fmt(element, f),
                Self::RollupTypeMap(element) => ::core::fmt::Display::fmt(element, f),
                Self::SetBatchFee(element) => ::core::fmt::Display::fmt(element, f),
                Self::SetMultiplierBatchFee(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::SetPendingStateTimeout(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::SetTrustedAggregatorTimeout(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::SetVerifyBatchTimeTarget(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::TotalSequencedBatches(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::TotalVerifiedBatches(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::TrustedAggregatorTimeout(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::UpdateRollup(element) => ::core::fmt::Display::fmt(element, f),
                Self::VerifyBatchTimeTarget(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::VerifyBatches(element) => ::core::fmt::Display::fmt(element, f),
                Self::VerifyBatchesTrustedAggregator(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
            }
        }
    }
    impl ::core::convert::From<DefaultAdminRoleCall> for PolygonRollupManagerCalls {
        fn from(value: DefaultAdminRoleCall) -> Self {
            Self::DefaultAdminRole(value)
        }
    }
    impl ::core::convert::From<ActivateEmergencyStateCall>
    for PolygonRollupManagerCalls {
        fn from(value: ActivateEmergencyStateCall) -> Self {
            Self::ActivateEmergencyState(value)
        }
    }
    impl ::core::convert::From<AddExistingRollupCall> for PolygonRollupManagerCalls {
        fn from(value: AddExistingRollupCall) -> Self {
            Self::AddExistingRollup(value)
        }
    }
    impl ::core::convert::From<AddNewRollupTypeCall> for PolygonRollupManagerCalls {
        fn from(value: AddNewRollupTypeCall) -> Self {
            Self::AddNewRollupType(value)
        }
    }
    impl ::core::convert::From<BridgeAddressCall> for PolygonRollupManagerCalls {
        fn from(value: BridgeAddressCall) -> Self {
            Self::BridgeAddress(value)
        }
    }
    impl ::core::convert::From<CalculateRewardPerBatchCall>
    for PolygonRollupManagerCalls {
        fn from(value: CalculateRewardPerBatchCall) -> Self {
            Self::CalculateRewardPerBatch(value)
        }
    }
    impl ::core::convert::From<ChainIDToRollupIDCall> for PolygonRollupManagerCalls {
        fn from(value: ChainIDToRollupIDCall) -> Self {
            Self::ChainIDToRollupID(value)
        }
    }
    impl ::core::convert::From<ConsolidatePendingStateCall>
    for PolygonRollupManagerCalls {
        fn from(value: ConsolidatePendingStateCall) -> Self {
            Self::ConsolidatePendingState(value)
        }
    }
    impl ::core::convert::From<CreateNewRollupCall> for PolygonRollupManagerCalls {
        fn from(value: CreateNewRollupCall) -> Self {
            Self::CreateNewRollup(value)
        }
    }
    impl ::core::convert::From<DeactivateEmergencyStateCall>
    for PolygonRollupManagerCalls {
        fn from(value: DeactivateEmergencyStateCall) -> Self {
            Self::DeactivateEmergencyState(value)
        }
    }
    impl ::core::convert::From<GetBatchFeeCall> for PolygonRollupManagerCalls {
        fn from(value: GetBatchFeeCall) -> Self {
            Self::GetBatchFee(value)
        }
    }
    impl ::core::convert::From<GetForcedBatchFeeCall> for PolygonRollupManagerCalls {
        fn from(value: GetForcedBatchFeeCall) -> Self {
            Self::GetForcedBatchFee(value)
        }
    }
    impl ::core::convert::From<GetInputSnarkBytesCall> for PolygonRollupManagerCalls {
        fn from(value: GetInputSnarkBytesCall) -> Self {
            Self::GetInputSnarkBytes(value)
        }
    }
    impl ::core::convert::From<GetLastVerifiedBatchCall> for PolygonRollupManagerCalls {
        fn from(value: GetLastVerifiedBatchCall) -> Self {
            Self::GetLastVerifiedBatch(value)
        }
    }
    impl ::core::convert::From<GetRoleAdminCall> for PolygonRollupManagerCalls {
        fn from(value: GetRoleAdminCall) -> Self {
            Self::GetRoleAdmin(value)
        }
    }
    impl ::core::convert::From<GetRollupBatchNumToStateRootCall>
    for PolygonRollupManagerCalls {
        fn from(value: GetRollupBatchNumToStateRootCall) -> Self {
            Self::GetRollupBatchNumToStateRoot(value)
        }
    }
    impl ::core::convert::From<GetRollupExitRootCall> for PolygonRollupManagerCalls {
        fn from(value: GetRollupExitRootCall) -> Self {
            Self::GetRollupExitRoot(value)
        }
    }
    impl ::core::convert::From<GetRollupPendingStateTransitionsCall>
    for PolygonRollupManagerCalls {
        fn from(value: GetRollupPendingStateTransitionsCall) -> Self {
            Self::GetRollupPendingStateTransitions(value)
        }
    }
    impl ::core::convert::From<GetRollupSequencedBatchesCall>
    for PolygonRollupManagerCalls {
        fn from(value: GetRollupSequencedBatchesCall) -> Self {
            Self::GetRollupSequencedBatches(value)
        }
    }
    impl ::core::convert::From<GlobalExitRootManagerCall> for PolygonRollupManagerCalls {
        fn from(value: GlobalExitRootManagerCall) -> Self {
            Self::GlobalExitRootManager(value)
        }
    }
    impl ::core::convert::From<GrantRoleCall> for PolygonRollupManagerCalls {
        fn from(value: GrantRoleCall) -> Self {
            Self::GrantRole(value)
        }
    }
    impl ::core::convert::From<HasRoleCall> for PolygonRollupManagerCalls {
        fn from(value: HasRoleCall) -> Self {
            Self::HasRole(value)
        }
    }
    impl ::core::convert::From<InitializeCall> for PolygonRollupManagerCalls {
        fn from(value: InitializeCall) -> Self {
            Self::Initialize(value)
        }
    }
    impl ::core::convert::From<IsEmergencyStateCall> for PolygonRollupManagerCalls {
        fn from(value: IsEmergencyStateCall) -> Self {
            Self::IsEmergencyState(value)
        }
    }
    impl ::core::convert::From<IsPendingStateConsolidableCall>
    for PolygonRollupManagerCalls {
        fn from(value: IsPendingStateConsolidableCall) -> Self {
            Self::IsPendingStateConsolidable(value)
        }
    }
    impl ::core::convert::From<LastAggregationTimestampCall>
    for PolygonRollupManagerCalls {
        fn from(value: LastAggregationTimestampCall) -> Self {
            Self::LastAggregationTimestamp(value)
        }
    }
    impl ::core::convert::From<LastDeactivatedEmergencyStateTimestampCall>
    for PolygonRollupManagerCalls {
        fn from(value: LastDeactivatedEmergencyStateTimestampCall) -> Self {
            Self::LastDeactivatedEmergencyStateTimestamp(value)
        }
    }
    impl ::core::convert::From<MultiplierBatchFeeCall> for PolygonRollupManagerCalls {
        fn from(value: MultiplierBatchFeeCall) -> Self {
            Self::MultiplierBatchFee(value)
        }
    }
    impl ::core::convert::From<ObsoleteRollupTypeCall> for PolygonRollupManagerCalls {
        fn from(value: ObsoleteRollupTypeCall) -> Self {
            Self::ObsoleteRollupType(value)
        }
    }
    impl ::core::convert::From<OnSequenceBatchesCall> for PolygonRollupManagerCalls {
        fn from(value: OnSequenceBatchesCall) -> Self {
            Self::OnSequenceBatches(value)
        }
    }
    impl ::core::convert::From<OverridePendingStateCall> for PolygonRollupManagerCalls {
        fn from(value: OverridePendingStateCall) -> Self {
            Self::OverridePendingState(value)
        }
    }
    impl ::core::convert::From<PendingStateTimeoutCall> for PolygonRollupManagerCalls {
        fn from(value: PendingStateTimeoutCall) -> Self {
            Self::PendingStateTimeout(value)
        }
    }
    impl ::core::convert::From<PolCall> for PolygonRollupManagerCalls {
        fn from(value: PolCall) -> Self {
            Self::Pol(value)
        }
    }
    impl ::core::convert::From<ProveNonDeterministicPendingStateCall>
    for PolygonRollupManagerCalls {
        fn from(value: ProveNonDeterministicPendingStateCall) -> Self {
            Self::ProveNonDeterministicPendingState(value)
        }
    }
    impl ::core::convert::From<RenounceRoleCall> for PolygonRollupManagerCalls {
        fn from(value: RenounceRoleCall) -> Self {
            Self::RenounceRole(value)
        }
    }
    impl ::core::convert::From<RevokeRoleCall> for PolygonRollupManagerCalls {
        fn from(value: RevokeRoleCall) -> Self {
            Self::RevokeRole(value)
        }
    }
    impl ::core::convert::From<RollupAddressToIDCall> for PolygonRollupManagerCalls {
        fn from(value: RollupAddressToIDCall) -> Self {
            Self::RollupAddressToID(value)
        }
    }
    impl ::core::convert::From<RollupCountCall> for PolygonRollupManagerCalls {
        fn from(value: RollupCountCall) -> Self {
            Self::RollupCount(value)
        }
    }
    impl ::core::convert::From<RollupIDToRollupDataCall> for PolygonRollupManagerCalls {
        fn from(value: RollupIDToRollupDataCall) -> Self {
            Self::RollupIDToRollupData(value)
        }
    }
    impl ::core::convert::From<RollupTypeCountCall> for PolygonRollupManagerCalls {
        fn from(value: RollupTypeCountCall) -> Self {
            Self::RollupTypeCount(value)
        }
    }
    impl ::core::convert::From<RollupTypeMapCall> for PolygonRollupManagerCalls {
        fn from(value: RollupTypeMapCall) -> Self {
            Self::RollupTypeMap(value)
        }
    }
    impl ::core::convert::From<SetBatchFeeCall> for PolygonRollupManagerCalls {
        fn from(value: SetBatchFeeCall) -> Self {
            Self::SetBatchFee(value)
        }
    }
    impl ::core::convert::From<SetMultiplierBatchFeeCall> for PolygonRollupManagerCalls {
        fn from(value: SetMultiplierBatchFeeCall) -> Self {
            Self::SetMultiplierBatchFee(value)
        }
    }
    impl ::core::convert::From<SetPendingStateTimeoutCall>
    for PolygonRollupManagerCalls {
        fn from(value: SetPendingStateTimeoutCall) -> Self {
            Self::SetPendingStateTimeout(value)
        }
    }
    impl ::core::convert::From<SetTrustedAggregatorTimeoutCall>
    for PolygonRollupManagerCalls {
        fn from(value: SetTrustedAggregatorTimeoutCall) -> Self {
            Self::SetTrustedAggregatorTimeout(value)
        }
    }
    impl ::core::convert::From<SetVerifyBatchTimeTargetCall>
    for PolygonRollupManagerCalls {
        fn from(value: SetVerifyBatchTimeTargetCall) -> Self {
            Self::SetVerifyBatchTimeTarget(value)
        }
    }
    impl ::core::convert::From<TotalSequencedBatchesCall> for PolygonRollupManagerCalls {
        fn from(value: TotalSequencedBatchesCall) -> Self {
            Self::TotalSequencedBatches(value)
        }
    }
    impl ::core::convert::From<TotalVerifiedBatchesCall> for PolygonRollupManagerCalls {
        fn from(value: TotalVerifiedBatchesCall) -> Self {
            Self::TotalVerifiedBatches(value)
        }
    }
    impl ::core::convert::From<TrustedAggregatorTimeoutCall>
    for PolygonRollupManagerCalls {
        fn from(value: TrustedAggregatorTimeoutCall) -> Self {
            Self::TrustedAggregatorTimeout(value)
        }
    }
    impl ::core::convert::From<UpdateRollupCall> for PolygonRollupManagerCalls {
        fn from(value: UpdateRollupCall) -> Self {
            Self::UpdateRollup(value)
        }
    }
    impl ::core::convert::From<VerifyBatchTimeTargetCall> for PolygonRollupManagerCalls {
        fn from(value: VerifyBatchTimeTargetCall) -> Self {
            Self::VerifyBatchTimeTarget(value)
        }
    }
    impl ::core::convert::From<VerifyBatchesCall> for PolygonRollupManagerCalls {
        fn from(value: VerifyBatchesCall) -> Self {
            Self::VerifyBatches(value)
        }
    }
    impl ::core::convert::From<VerifyBatchesTrustedAggregatorCall>
    for PolygonRollupManagerCalls {
        fn from(value: VerifyBatchesTrustedAggregatorCall) -> Self {
            Self::VerifyBatchesTrustedAggregator(value)
        }
    }
    ///Container type for all return fields from the `DEFAULT_ADMIN_ROLE` function with signature `DEFAULT_ADMIN_ROLE()` and selector `0xa217fddf`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct DefaultAdminRoleReturn(pub [u8; 32]);
    ///Container type for all return fields from the `bridgeAddress` function with signature `bridgeAddress()` and selector `0xa3c573eb`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct BridgeAddressReturn(pub ::ethers::core::types::Address);
    ///Container type for all return fields from the `calculateRewardPerBatch` function with signature `calculateRewardPerBatch()` and selector `0x99f5634e`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct CalculateRewardPerBatchReturn(pub ::ethers::core::types::U256);
    ///Container type for all return fields from the `chainIDToRollupID` function with signature `chainIDToRollupID(uint64)` and selector `0x7fb6e76a`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct ChainIDToRollupIDReturn {
        pub rollup_id: u32,
    }
    ///Container type for all return fields from the `getBatchFee` function with signature `getBatchFee()` and selector `0x477fa270`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct GetBatchFeeReturn(pub ::ethers::core::types::U256);
    ///Container type for all return fields from the `getForcedBatchFee` function with signature `getForcedBatchFee()` and selector `0x60469169`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct GetForcedBatchFeeReturn(pub ::ethers::core::types::U256);
    ///Container type for all return fields from the `getInputSnarkBytes` function with signature `getInputSnarkBytes(uint32,uint64,uint64,bytes32,bytes32,bytes32)` and selector `0x7975fcfe`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct GetInputSnarkBytesReturn(pub ::ethers::core::types::Bytes);
    ///Container type for all return fields from the `getLastVerifiedBatch` function with signature `getLastVerifiedBatch(uint32)` and selector `0x11f6b287`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct GetLastVerifiedBatchReturn(pub u64);
    ///Container type for all return fields from the `getRoleAdmin` function with signature `getRoleAdmin(bytes32)` and selector `0x248a9ca3`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct GetRoleAdminReturn(pub [u8; 32]);
    ///Container type for all return fields from the `getRollupBatchNumToStateRoot` function with signature `getRollupBatchNumToStateRoot(uint32,uint64)` and selector `0x55a71ee0`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct GetRollupBatchNumToStateRootReturn(pub [u8; 32]);
    ///Container type for all return fields from the `getRollupExitRoot` function with signature `getRollupExitRoot()` and selector `0xa2967d99`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct GetRollupExitRootReturn(pub [u8; 32]);
    ///Container type for all return fields from the `getRollupPendingStateTransitions` function with signature `getRollupPendingStateTransitions(uint32,uint64)` and selector `0xb99d0ad7`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct GetRollupPendingStateTransitionsReturn(pub PendingState);
    ///Container type for all return fields from the `getRollupSequencedBatches` function with signature `getRollupSequencedBatches(uint32,uint64)` and selector `0x25280169`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct GetRollupSequencedBatchesReturn(pub SequencedBatchData);
    ///Container type for all return fields from the `globalExitRootManager` function with signature `globalExitRootManager()` and selector `0xd02103ca`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct GlobalExitRootManagerReturn(pub ::ethers::core::types::Address);
    ///Container type for all return fields from the `hasRole` function with signature `hasRole(bytes32,address)` and selector `0x91d14854`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct HasRoleReturn(pub bool);
    ///Container type for all return fields from the `isEmergencyState` function with signature `isEmergencyState()` and selector `0x15064c96`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct IsEmergencyStateReturn(pub bool);
    ///Container type for all return fields from the `isPendingStateConsolidable` function with signature `isPendingStateConsolidable(uint32,uint64)` and selector `0x080b3111`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct IsPendingStateConsolidableReturn(pub bool);
    ///Container type for all return fields from the `lastAggregationTimestamp` function with signature `lastAggregationTimestamp()` and selector `0xc1acbc34`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct LastAggregationTimestampReturn(pub u64);
    ///Container type for all return fields from the `lastDeactivatedEmergencyStateTimestamp` function with signature `lastDeactivatedEmergencyStateTimestamp()` and selector `0x30c27dde`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct LastDeactivatedEmergencyStateTimestampReturn(pub u64);
    ///Container type for all return fields from the `multiplierBatchFee` function with signature `multiplierBatchFee()` and selector `0xafd23cbe`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct MultiplierBatchFeeReturn(pub u16);
    ///Container type for all return fields from the `onSequenceBatches` function with signature `onSequenceBatches(uint64,bytes32)` and selector `0x9a908e73`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct OnSequenceBatchesReturn(pub u64);
    ///Container type for all return fields from the `pendingStateTimeout` function with signature `pendingStateTimeout()` and selector `0xd939b315`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct PendingStateTimeoutReturn(pub u64);
    ///Container type for all return fields from the `pol` function with signature `pol()` and selector `0xe46761c4`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct PolReturn(pub ::ethers::core::types::Address);
    ///Container type for all return fields from the `rollupAddressToID` function with signature `rollupAddressToID(address)` and selector `0xceee281d`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct RollupAddressToIDReturn {
        pub rollup_id: u32,
    }
    ///Container type for all return fields from the `rollupCount` function with signature `rollupCount()` and selector `0xf4e92675`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct RollupCountReturn(pub u32);
    ///Container type for all return fields from the `rollupIDToRollupData` function with signature `rollupIDToRollupData(uint32)` and selector `0xf9c4c2ae`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct RollupIDToRollupDataReturn {
        pub rollup_contract: ::ethers::core::types::Address,
        pub chain_id: u64,
        pub verifier: ::ethers::core::types::Address,
        pub fork_id: u64,
        pub last_local_exit_root: [u8; 32],
        pub last_batch_sequenced: u64,
        pub last_verified_batch: u64,
        pub last_pending_state: u64,
        pub last_pending_state_consolidated: u64,
        pub last_verified_batch_before_upgrade: u64,
        pub rollup_type_id: u64,
        pub rollup_compatibility_id: u8,
    }
    ///Container type for all return fields from the `rollupTypeCount` function with signature `rollupTypeCount()` and selector `0x1796a1ae`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct RollupTypeCountReturn(pub u32);
    ///Container type for all return fields from the `rollupTypeMap` function with signature `rollupTypeMap(uint32)` and selector `0x65c0504d`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct RollupTypeMapReturn {
        pub consensus_implementation: ::ethers::core::types::Address,
        pub verifier: ::ethers::core::types::Address,
        pub fork_id: u64,
        pub rollup_compatibility_id: u8,
        pub obsolete: bool,
        pub genesis: [u8; 32],
    }
    ///Container type for all return fields from the `totalSequencedBatches` function with signature `totalSequencedBatches()` and selector `0x066ec012`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct TotalSequencedBatchesReturn(pub u64);
    ///Container type for all return fields from the `totalVerifiedBatches` function with signature `totalVerifiedBatches()` and selector `0xdde0ff77`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct TotalVerifiedBatchesReturn(pub u64);
    ///Container type for all return fields from the `trustedAggregatorTimeout` function with signature `trustedAggregatorTimeout()` and selector `0x841b24d7`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct TrustedAggregatorTimeoutReturn(pub u64);
    ///Container type for all return fields from the `verifyBatchTimeTarget` function with signature `verifyBatchTimeTarget()` and selector `0x0a0d9fbe`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct VerifyBatchTimeTargetReturn(pub u64);
    ///`PendingState(uint64,uint64,bytes32,bytes32)`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct PendingState {
        pub timestamp: u64,
        pub last_verified_batch: u64,
        pub exit_root: [u8; 32],
        pub state_root: [u8; 32],
    }
    ///`SequencedBatchData(bytes32,uint64,uint64)`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct SequencedBatchData {
        pub acc_input_hash: [u8; 32],
        pub sequenced_timestamp: u64,
        pub previous_last_batch_sequenced: u64,
    }
}
