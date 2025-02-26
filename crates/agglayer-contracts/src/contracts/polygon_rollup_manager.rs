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
                    ::ethers::core::abi::ethabi::Param {
                        name: ::std::borrow::ToOwned::to_owned("_aggLayerGateway"),
                        kind: ::ethers::core::abi::ethabi::ParamType::Address,
                        internal_type: ::core::option::Option::Some(
                            ::std::borrow::ToOwned::to_owned("contract AggLayerGateway"),
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
                    ::std::borrow::ToOwned::to_owned("ROLLUP_MANAGER_VERSION"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "ROLLUP_MANAGER_VERSION",
                            ),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
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
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("verifier"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
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
                                    name: ::std::borrow::ToOwned::to_owned("initRoot"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "rollupVerifierType",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "enum IPolygonRollupManager.VerifierType",
                                        ),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("programVKey"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "initPessimisticRoot",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
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
                                        ::std::borrow::ToOwned::to_owned("address"),
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
                                        "rollupVerifierType",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "enum IPolygonRollupManager.VerifierType",
                                        ),
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
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("programVKey"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
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
                    ::std::borrow::ToOwned::to_owned("aggLayerGateway"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("aggLayerGateway"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("contract AggLayerGateway"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
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
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "initializeBytesCustomChain",
                                    ),
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
                    ::std::borrow::ToOwned::to_owned("getInputPessimisticBytes"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "getInputPessimisticBytes",
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
                                    name: ::std::borrow::ToOwned::to_owned("l1InfoTreeRoot"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
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
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "newPessimisticRoot",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("customChainData"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes"),
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
                            inputs: ::std::vec![],
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
                    ::std::borrow::ToOwned::to_owned("rollbackBatches"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("rollbackBatches"),
                            inputs: ::std::vec![
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
                                    name: ::std::borrow::ToOwned::to_owned("targetBatch"),
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
                                    name: ::std::borrow::ToOwned::to_owned("rollupData"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Address,
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::Address,
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "struct PolygonRollupManager.RollupDataReturn",
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
                    ::std::borrow::ToOwned::to_owned("rollupIDToRollupDataDeserialized"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "rollupIDToRollupDataDeserialized",
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
                                        ::std::borrow::ToOwned::to_owned("address"),
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
                                        ::std::borrow::ToOwned::to_owned("address"),
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
                                        "rollupVerifierType",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "enum IPolygonRollupManager.VerifierType",
                                        ),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "lastPessimisticRoot",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("programVKey"),
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
                    ::std::borrow::ToOwned::to_owned("rollupIDToRollupDataV2"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "rollupIDToRollupDataV2",
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
                                    name: ::std::borrow::ToOwned::to_owned("rollupData"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Address,
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::Address,
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "struct PolygonRollupManager.RollupDataReturnV2",
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
                                        ::std::borrow::ToOwned::to_owned("address"),
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
                                        "rollupVerifierType",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "enum IPolygonRollupManager.VerifierType",
                                        ),
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
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("programVKey"),
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
                    ::std::borrow::ToOwned::to_owned("updateRollupByRollupAdmin"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "updateRollupByRollupAdmin",
                            ),
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
                (
                    ::std::borrow::ToOwned::to_owned(
                        "verifyPessimisticTrustedAggregator",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "verifyPessimisticTrustedAggregator",
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
                                        "l1InfoTreeLeafCount",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint32"),
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
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "newPessimisticRoot",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("proof"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("customChainData"),
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
                                        "rollupVerifierType",
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
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("programVKey"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "initPessimisticRoot",
                                    ),
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
                                        "rollupVerifierType",
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
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("programVKey"),
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
                    ::std::borrow::ToOwned::to_owned("RollbackBatches"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("RollbackBatches"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("rollupID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    indexed: true,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("targetBatch"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    indexed: true,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "accInputHashToRollback",
                                    ),
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
                    ::std::borrow::ToOwned::to_owned("UpdateRollupManagerVersion"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned(
                                "UpdateRollupManagerVersion",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "rollupManagerVersion",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    indexed: false,
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
                    ::std::borrow::ToOwned::to_owned("AllBatchesMustBeVerified"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "AllBatchesMustBeVerified",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("AllSequencedMustBeVerified"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "AllSequencedMustBeVerified",
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
                    ::std::borrow::ToOwned::to_owned(
                        "CannotUpdateWithUnconsolidatedPendingState",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "CannotUpdateWithUnconsolidatedPendingState",
                            ),
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
                    ::std::borrow::ToOwned::to_owned("ChainIDOutOfRange"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("ChainIDOutOfRange"),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("EmptyVerifySequencesData"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "EmptyVerifySequencesData",
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
                    ::std::borrow::ToOwned::to_owned(
                        "FinalNumSequenceBelowLastVerifiedSequence",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "FinalNumSequenceBelowLastVerifiedSequence",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned(
                        "FinalNumSequenceDoesNotMatchPendingState",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "FinalNumSequenceDoesNotMatchPendingState",
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
                    ::std::borrow::ToOwned::to_owned(
                        "InitSequenceMustMatchCurrentForkID",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "InitSequenceMustMatchCurrentForkID",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned(
                        "InitSequenceNumDoesNotMatchPendingState",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "InitSequenceNumDoesNotMatchPendingState",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("InvalidPessimisticProof"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "InvalidPessimisticProof",
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
                    ::std::borrow::ToOwned::to_owned("InvalidRangeMultiplierZkGasPrice"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "InvalidRangeMultiplierZkGasPrice",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("InvalidRangeSequenceTimeTarget"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "InvalidRangeSequenceTimeTarget",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("InvalidRollup"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("InvalidRollup"),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("InvalidRollupType"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("InvalidRollupType"),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("InvalidVerifierType"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "InvalidVerifierType",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("L1InfoTreeLeafCountInvalid"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "L1InfoTreeLeafCountInvalid",
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
                    ::std::borrow::ToOwned::to_owned("MustSequenceSomeBlob"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "MustSequenceSomeBlob",
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
                    ::std::borrow::ToOwned::to_owned("NotAllowedAddress"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("NotAllowedAddress"),
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
                    ::std::borrow::ToOwned::to_owned("OnlyRollupAdmin"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("OnlyRollupAdmin"),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("OnlyStateTransitionChains"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "OnlyStateTransitionChains",
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
                    ::std::borrow::ToOwned::to_owned("PendingStateNumExist"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "PendingStateNumExist",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("ReentrancyGuardReentrantCall"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "ReentrancyGuardReentrantCall",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("RollbackBatchIsNotEndOfSequence"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "RollbackBatchIsNotEndOfSequence",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("RollbackBatchIsNotValid"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "RollbackBatchIsNotValid",
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
                    ::std::borrow::ToOwned::to_owned("RollupIDNotAscendingOrder"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "RollupIDNotAscendingOrder",
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
                    ::std::borrow::ToOwned::to_owned("StateTransitionChainsNotAllowed"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "StateTransitionChainsNotAllowed",
                            ),
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
                    ::std::borrow::ToOwned::to_owned("UpdateToOldRollupTypeID"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "UpdateToOldRollupTypeID",
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
                (
                    ::std::borrow::ToOwned::to_owned("zkGasPriceOfRange"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("zkGasPriceOfRange"),
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
    #[rustfmt::skip]
    const __BYTECODE: &[u8] = b"a\x01\0`@R4\x80\x15a\0\x10W__\xFD[P`@QaX^8\x03\x80aX^\x839\x81\x01`@\x81\x90Ra\0/\x91a\x013V[`\x01`\x01`\xA0\x1B\x03\x80\x85\x16`\x80R\x83\x81\x16`\xC0R\x82\x81\x16`\xA0R\x81\x16`\xE0Ra\0Va\0_V[PPPPa\x01\x8FV[_Ta\x01\0\x90\x04`\xFF\x16\x15a\0\xCAW`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`'`$\x82\x01R\x7FInitializable: contract is initi`D\x82\x01Rfalizing`\xC8\x1B`d\x82\x01R`\x84\x01`@Q\x80\x91\x03\x90\xFD[_T`\xFF\x90\x81\x16\x10\x15a\x01\x1AW_\x80T`\xFF\x19\x16`\xFF\x90\x81\x17\x90\x91U`@Q\x90\x81R\x7F\x7F&\xB8?\xF9n\x1F+jh/\x138R\xF6y\x8A\t\xC4e\xDA\x95\x92\x14`\xCE\xFB8G@$\x98\x90` \x01`@Q\x80\x91\x03\x90\xA1[V[`\x01`\x01`\xA0\x1B\x03\x81\x16\x81\x14a\x010W__\xFD[PV[____`\x80\x85\x87\x03\x12\x15a\x01FW__\xFD[\x84Qa\x01Q\x81a\x01\x1CV[` \x86\x01Q\x90\x94Pa\x01b\x81a\x01\x1CV[`@\x86\x01Q\x90\x93Pa\x01s\x81a\x01\x1CV[``\x86\x01Q\x90\x92Pa\x01\x84\x81a\x01\x1CV[\x93\x96\x92\x95P\x90\x93PPV[`\x80Q`\xA0Q`\xC0Q`\xE0QaVda\x01\xFA_9_\x81\x81a\x06\xCB\x01Ra\x0E\x14\x01R_\x81\x81a\x08'\x01R\x81\x81a\x17\xD8\x01Ra0\xF5\x01R_\x81\x81a\x06\x8C\x01R\x81\x81a'P\x01Ra1\xF3\x01R_\x81\x81a\x07j\x01R\x81\x81a\nj\x01R\x81\x81a\r3\x01Ra\x0F\x1B\x01RaVd_\xF3\xFE`\x80`@R4\x80\x15a\0\x0FW__\xFD[P`\x046\x10a\x02\xF5W_5`\xE0\x1C\x80c\x99\xF5cN\x11a\x01\x9DW\x80c\xD5\x07?o\x11a\0\xE8W\x80c\xDF\xDB\x8C^\x11a\0\x93W\x80c\xE8\x0EP0\x11a\0nW\x80c\xE8\x0EP0\x14a\x08\xF7W\x80c\xF4\xE9&u\x14a\t\nW\x80c\xF9\xC4\xC2\xAE\x14a\t\x1AW__\xFD[\x80c\xDF\xDB\x8C^\x14a\x08\x0FW\x80c\xE4ga\xC4\x14a\x08\"W\x80c\xE4\xF3\xD8\xF9\x14a\x08IW__\xFD[\x80c\xDB\xC1iv\x11a\0\xC3W\x80c\xDB\xC1iv\x14a\x07\xDAW\x80c\xDD\x04d\xB9\x14a\x07\xE2W\x80c\xDD\xE0\xFFw\x14a\x07\xF5W__\xFD[\x80c\xD5\x07?o\x14a\x07\x8CW\x80c\xD5Gt\x1F\x14a\x07\x9FW\x80c\xD8\x90X\x12\x14a\x07\xB2W__\xFD[\x80c\xAB\xCBQ\x98\x11a\x01HW\x80c\xC5\xB4\xFD\xB6\x11a\x01#W\x80c\xC5\xB4\xFD\xB6\x14a\x07-W\x80c\xCE\xEE(\x1D\x14a\x07@W\x80c\xD0!\x03\xCA\x14a\x07eW__\xFD[\x80c\xAB\xCBQ\x98\x14a\x06\xEDW\x80c\xC1\xAC\xBC4\x14a\x07\0W\x80c\xC4\xC9(\xC2\x14a\x07\x1AW__\xFD[\x80c\xA2\x96}\x99\x11a\x01xW\x80c\xA2\x96}\x99\x14a\x06\x7FW\x80c\xA3\xC5s\xEB\x14a\x06\x87W\x80c\xAB\x04u\xCF\x14a\x06\xC6W__\xFD[\x80c\x99\xF5cN\x14a\x06]W\x80c\x9A\x90\x8Es\x14a\x06eW\x80c\xA2\x17\xFD\xDF\x14a\x06xW__\xFD[\x80cG\x7F\xA2p\x11a\x02]W\x80ct\xD9\xC2D\x11a\x02\x08W\x80c\x81)\xFC\x1C\x11a\x01\xE3W\x80c\x81)\xFC\x1C\x14a\x06\nW\x80c\x8F\xD8\x8C\xC2\x14a\x06\x12W\x80c\x91\xD1HT\x14a\x06%W__\xFD[\x80ct\xD9\xC2D\x14a\x05\xA5W\x80cyu\xFC\xFE\x14a\x05\xC5W\x80c\x7F\xB6\xE7j\x14a\x05\xE5W__\xFD[\x80ce\xC0PM\x11a\x028W\x80ce\xC0PM\x14a\x05\x04W\x80clvhw\x14a\x05\x7FW\x80cr\"\x02\x0F\x14a\x05\x92W__\xFD[\x80cG\x7F\xA2p\x14a\x04\xB4W\x80cU\xA7\x1E\xE0\x14a\x04\xBCW\x80c`F\x91i\x14a\x04\xFCW__\xFD[\x80c r\xF6\xC5\x11a\x02\xBDW\x80c//\xF1]\x11a\x02\x98W\x80c//\xF1]\x14a\x04{W\x80c0\xC2}\xDE\x14a\x04\x8EW\x80c6V\x8A\xBE\x14a\x04\xA1W__\xFD[\x80c r\xF6\xC5\x14a\x03\x93W\x80c$\x8A\x9C\xA3\x14a\x03\x9BW\x80c%(\x01i\x14a\x03\xCBW__\xFD[\x80c\x06n\xC0\x12\x14a\x02\xF9W\x80c\x11\xF6\xB2\x87\x14a\x03)W\x80c\x14\x89\xED\x10\x14a\x03<W\x80c\x15\x06L\x96\x14a\x03QW\x80c\x17\x96\xA1\xAE\x14a\x03nW[__\xFD[`\x84Ta\x03\x0C\x90`\x01`\x01`@\x1B\x03\x16\x81V[`@Q`\x01`\x01`@\x1B\x03\x90\x91\x16\x81R` \x01[`@Q\x80\x91\x03\x90\xF3[a\x03\x0Ca\x0376`\x04a>\xC0V[a\t:V[a\x03Oa\x03J6`\x04a?\x15V[a\tiV[\0[`oTa\x03^\x90`\xFF\x16\x81V[`@Q\x90\x15\x15\x81R` \x01a\x03 V[`~Ta\x03~\x90c\xFF\xFF\xFF\xFF\x16\x81V[`@Qc\xFF\xFF\xFF\xFF\x90\x91\x16\x81R` \x01a\x03 V[a\x03Oa\x0BJV[a\x03\xBDa\x03\xA96`\x04a?\xA3V[_\x90\x81R`4` R`@\x90 `\x01\x01T\x90V[`@Q\x90\x81R` \x01a\x03 V[a\x04Ha\x03\xD96`\x04a?\xBAV[`@\x80Q``\x80\x82\x01\x83R_\x80\x83R` \x80\x84\x01\x82\x90R\x92\x84\x01\x81\x90Rc\xFF\xFF\xFF\xFF\x95\x90\x95\x16\x85R`\x81\x82R\x82\x85 `\x01`\x01`@\x1B\x03\x94\x85\x16\x86R`\x03\x01\x82R\x93\x82\x90 \x82Q\x94\x85\x01\x83R\x80T\x85R`\x01\x01T\x80\x84\x16\x91\x85\x01\x91\x90\x91R`\x01`@\x1B\x90\x04\x90\x91\x16\x90\x82\x01R\x90V[`@\x80Q\x82Q\x81R` \x80\x84\x01Q`\x01`\x01`@\x1B\x03\x90\x81\x16\x91\x83\x01\x91\x90\x91R\x92\x82\x01Q\x90\x92\x16\x90\x82\x01R``\x01a\x03 V[a\x03Oa\x04\x896`\x04a?\xEBV[a\x0C\x1CV[`\x87Ta\x03\x0C\x90`\x01`\x01`@\x1B\x03\x16\x81V[a\x03Oa\x04\xAF6`\x04a?\xEBV[a\x0CEV[`\x86Ta\x03\xBDV[a\x03\xBDa\x04\xCA6`\x04a?\xBAV[c\xFF\xFF\xFF\xFF\x82\x16_\x90\x81R`\x81` \x90\x81R`@\x80\x83 `\x01`\x01`@\x1B\x03\x85\x16\x84R`\x02\x01\x90\x91R\x90 T\x92\x91PPV[a\x03\xBDa\x0C|V[a\x05la\x05\x126`\x04a>\xC0V[`\x7F` R_\x90\x81R`@\x90 \x80T`\x01\x82\x01T`\x02\x83\x01T`\x03\x90\x93\x01T`\x01`\x01`\xA0\x1B\x03\x92\x83\x16\x93\x92\x82\x16\x92`\x01`\x01`@\x1B\x03`\x01`\xA0\x1B\x84\x04\x16\x92`\xFF`\x01`\xE0\x1B\x82\x04\x81\x16\x93`\x01`\xE8\x1B\x90\x92\x04\x16\x91\x90\x87V[`@Qa\x03 \x97\x96\x95\x94\x93\x92\x91\x90a@MV[a\x03Oa\x05\x8D6`\x04a@\xE3V[a\x0C\x91V[a\x03Oa\x05\xA06`\x04a>\xC0V[a\x10\x81V[a\x05\xB8a\x05\xB36`\x04a>\xC0V[a\x11rV[`@Qa\x03 \x91\x90aA\x83V[a\x05\xD8a\x05\xD36`\x04aB\x91V[a\x12\xC1V[`@Qa\x03 \x91\x90aC\x1AV[a\x03~a\x05\xF36`\x04aC,V[`\x83` R_\x90\x81R`@\x90 Tc\xFF\xFF\xFF\xFF\x16\x81V[a\x03Oa\x12\xF1V[a\x03Oa\x06 6`\x04aCEV[a\x142V[a\x03^a\x0636`\x04a?\xEBV[_\x91\x82R`4` \x90\x81R`@\x80\x84 `\x01`\x01`\xA0\x1B\x03\x93\x90\x93\x16\x84R\x91\x90R\x90 T`\xFF\x16\x90V[a\x03\xBDa\x17\xB7V[a\x03\x0Ca\x06s6`\x04aCaV[a\x18\x90V[a\x03\xBD_\x81V[a\x03\xBDa\x1A\x86V[a\x06\xAE\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81V[`@Q`\x01`\x01`\xA0\x1B\x03\x90\x91\x16\x81R` \x01a\x03 V[a\x06\xAE\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81V[a\x03Oa\x06\xFB6`\x04aD:V[a\x1D\xECV[`\x84Ta\x03\x0C\x90`\x01`\x80\x1B\x90\x04`\x01`\x01`@\x1B\x03\x16\x81V[a\x03Oa\x07(6`\x04aD\xCDV[a \xE4V[a\x03Oa\x07;6`\x04aE(V[a!\x1FV[a\x03~a\x07N6`\x04aF\x05V[`\x82` R_\x90\x81R`@\x90 Tc\xFF\xFF\xFF\xFF\x16\x81V[a\x06\xAE\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81V[a\x03Oa\x07\x9A6`\x04a?\xA3V[a&6V[a\x03Oa\x07\xAD6`\x04a?\xEBV[a&\xD4V[a\x05\xD8`@Q\x80`@\x01`@R\x80`\t\x81R` \x01h\x06\x16\xC2\xD7c\x02\xE32\xE3`\xBC\x1B\x81RP\x81V[a\x03Oa&\xF8V[a\x05\xD8a\x07\xF06`\x04aF V[a'\xBEV[`\x84Ta\x03\x0C\x90`\x01`@\x1B\x90\x04`\x01`\x01`@\x1B\x03\x16\x81V[a\x03Oa\x08\x1D6`\x04aF\x8BV[a'\xE5V[a\x06\xAE\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81V[a\x08\xDFa\x08W6`\x04a>\xC0V[c\xFF\xFF\xFF\xFF\x16_\x90\x81R`\x81` R`@\x90 \x80T`\x01\x82\x01T`\x05\x83\x01T`\x06\x84\x01T`\x07\x85\x01T`\x08\x86\x01T`\t\x90\x96\x01T`\x01`\x01`\xA0\x1B\x03\x80\x87\x16\x98`\x01`\xA0\x1B\x97\x88\x90\x04`\x01`\x01`@\x1B\x03\x90\x81\x16\x99\x92\x88\x16\x98\x90\x97\x04\x87\x16\x96\x80\x86\x16\x95`\x01`@\x1B\x90\x81\x90\x04\x82\x16\x95\x82\x81\x16\x95\x91\x81\x04\x90\x92\x16\x93`\x01`\x80\x1B\x90\x92\x04`\xFF\x16\x92V[`@Qa\x03 \x9C\x9B\x9A\x99\x98\x97\x96\x95\x94\x93\x92\x91\x90aF\xB5V[a\x03Oa\t\x056`\x04aGFV[a*\rV[`\x80Ta\x03~\x90c\xFF\xFF\xFF\xFF\x16\x81V[a\t-a\t(6`\x04a>\xC0V[a-CV[`@Qa\x03 \x91\x90aG\xC7V[c\xFF\xFF\xFF\xFF\x81\x16_\x90\x81R`\x81` R`@\x81 `\x06\x01T`\x01`@\x1B\x90\x04`\x01`\x01`@\x1B\x03\x16[\x92\x91PPV[\x7F\x08N\x94\xF3u\xE9\xD6G\xF8\x7F[,\xEF\xFB\xA1\xE0b\xC7\x0F`\t\xFD\xBC\xF8\x02\x91\xE8\x03\xB5\xC9\xED\xD4a\t\x93\x81a.\x99V[`\x01`\x01`@\x1B\x03\x88\x16\x15a\t\xBBW`@Qc0m\xFCW`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[c\xFF\xFF\xFF\xFF\x89\x16_\x90\x81R`\x81` R`@\x81 \x90`\x07\x82\x01T`\x01`\x80\x1B\x90\x04`\xFF\x16`\x02\x81\x11\x15a\t\xF0Wa\t\xF0a@\x19V[\x14a\n\x0EW`@Qc\x90\xDB\r\x07`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\n\x1D\x81\x89\x89\x89\x89\x89\x89a.\xA3V[`\x06\x81\x01\x80To\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\x19\x16`\x01`@\x1B`\x01`\x01`@\x1B\x03\x8A\x16\x90\x81\x02\x91\x90\x91\x17\x90\x91U_\x90\x81R`\x02\x82\x01` R`@\x90 \x85\x90U`\x05\x81\x01\x86\x90U\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`\x01`\x01`\xA0\x1B\x03\x16c3\xD6$}a\n\x9Fa\x1A\x86V[`@Q\x82c\xFF\xFF\xFF\xFF\x16`\xE0\x1B\x81R`\x04\x01a\n\xBD\x91\x81R` \x01\x90V[_`@Q\x80\x83\x03\x81_\x87\x80;\x15\x80\x15a\n\xD4W__\xFD[PZ\xF1\x15\x80\x15a\n\xE6W=__>=_\xFD[PP`@\x80Q`\x01`\x01`@\x1B\x03\x8B\x16\x81R` \x81\x01\x89\x90R\x90\x81\x01\x89\x90R3\x92Pc\xFF\xFF\xFF\xFF\x8D\x16\x91P\x7F\xD1\xEC:\x12\x16\xF0\x8Bn\xFFr\xE1i\xCE\xB5H\xB7\x82\xDB\x18\xA6aHRa\x8D\x86\xBB\x19\xF3\xF9\xB0\xD3\x90``\x01`@Q\x80\x91\x03\x90\xA3PPPPPPPPPPV[3_\x90\x81R\x7F\x88u\xB9J\xF5ez)\x03\xDE\xF9\x90mg\xA3\xF4-\x8A\x83m$\xB5`,\0\xF0\x0F\xC8U3\x9F\xCD` R`@\x90 T`\xFF\x16a\x0C\x12W`\x84T`\x01`\x80\x1B\x90\x04`\x01`\x01`@\x1B\x03\x16\x15\x80a\x0B\xC7WP`\x84TB\x90a\x0B\xBC\x90b\t:\x80\x90`\x01`\x80\x1B\x90\x04`\x01`\x01`@\x1B\x03\x16aI\x0BV[`\x01`\x01`@\x1B\x03\x16\x11[\x80a\x0B\xF4WP`\x87TB\x90a\x0B\xE9\x90b\t:\x80\x90`\x01`\x01`@\x1B\x03\x16aI\x0BV[`\x01`\x01`@\x1B\x03\x16\x11[\x15a\x0C\x12W`@Qci+\xAA\xAD`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\x0C\x1Aa1\xF1V[V[_\x82\x81R`4` R`@\x90 `\x01\x01Ta\x0C6\x81a.\x99V[a\x0C@\x83\x83a2gV[PPPV[`\x01`\x01`\xA0\x1B\x03\x81\x163\x14a\x0CnW`@Qc\x0BJ\xD1\xCD`\xE3\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\x0Cx\x82\x82a2\xEAV[PPV[_`\x86T`da\x0C\x8C\x91\x90aI*V[\x90P\x90V[\x7F\x08N\x94\xF3u\xE9\xD6G\xF8\x7F[,\xEF\xFB\xA1\xE0b\xC7\x0F`\t\xFD\xBC\xF8\x02\x91\xE8\x03\xB5\xC9\xED\xD4a\x0C\xBB\x81a.\x99V[a\x0C\xC3a3kV[c\xFF\xFF\xFF\xFF\x89\x16_\x90\x81R`\x81` R`@\x81 \x90`\x07\x82\x01T`\x01`\x80\x1B\x90\x04`\xFF\x16`\x02\x81\x11\x15a\x0C\xF8Wa\x0C\xF8a@\x19V[\x03a\r\x16W`@Qc[f\x02\xB7`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`@Qc\xEFN\xEB5`\xE0\x1B\x81Rc\xFF\xFF\xFF\xFF\x8A\x16`\x04\x82\x01R_\x90\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`\x01`\x01`\xA0\x1B\x03\x16\x90c\xEFN\xEB5\x90`$\x01` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a\r\x80W=__>=_\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\r\xA4\x91\x90aIAV[\x90P\x80a\r\xC4W`@Qc\xA6\x07!\xE1`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[_a\r\xD4\x8C\x84\x84\x8D\x8D\x8B\x8Ba3\xD8V[\x90P`\x02`\x07\x84\x01T`\x01`\x80\x1B\x90\x04`\xFF\x16`\x02\x81\x11\x15a\r\xF8Wa\r\xF8a@\x19V[\x03a\x0E~W`@Qc\xA4\x8F\xD3w`\xE0\x1B\x81R`\x01`\x01`\xA0\x1B\x03\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x16\x90c\xA4\x8F\xD3w\x90a\x0EM\x90\x84\x90\x8C\x90\x8C\x90`\x04\x01aI\x80V[_`@Q\x80\x83\x03\x81\x86\x80;\x15\x80\x15a\x0EcW__\xFD[PZ\xFA\x15\x80\x15a\x0EuW=__>=_\xFD[PPPPa\x0E\xE6V[`\x01\x83\x01T`\t\x84\x01T`@Qc\x02\nI\xE3`\xE5\x1B\x81R`\x01`\x01`\xA0\x1B\x03\x90\x92\x16\x91cAI<`\x91a\x0E\xB9\x91\x85\x90\x8D\x90\x8D\x90`\x04\x01aI\xAFV[_`@Q\x80\x83\x03\x81\x86\x80;\x15\x80\x15a\x0E\xCFW__\xFD[PZ\xFA\x15\x80\x15a\x0E\xE1W=__>=_\xFD[PPPP[`\x84\x80Tg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF`\x80\x1B\x19\x16`\x01`\x80\x1BB`\x01`\x01`@\x1B\x03\x16\x02\x17\x90U`\x05\x83\x01\x8A\x90U`\x08\x83\x01\x89\x90U\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`\x01`\x01`\xA0\x1B\x03\x16c3\xD6$}a\x0FPa\x1A\x86V[`@Q\x82c\xFF\xFF\xFF\xFF\x16`\xE0\x1B\x81R`\x04\x01a\x0Fn\x91\x81R` \x01\x90V[_`@Q\x80\x83\x03\x81_\x87\x80;\x15\x80\x15a\x0F\x85W__\xFD[PZ\xF1\x15\x80\x15a\x0F\x97W=__>=_\xFD[PP`@\x80Q_\x80\x82R` \x82\x01R\x90\x81\x01\x8D\x90R3\x92Pc\xFF\xFF\xFF\xFF\x8F\x16\x91P\x7F\xD1\xEC:\x12\x16\xF0\x8Bn\xFFr\xE1i\xCE\xB5H\xB7\x82\xDB\x18\xA6aHRa\x8D\x86\xBB\x19\xF3\xF9\xB0\xD3\x90``\x01`@Q\x80\x91\x03\x90\xA3`\x02`\x07\x84\x01T`\x01`\x80\x1B\x90\x04`\xFF\x16`\x02\x81\x11\x15a\x10\x07Wa\x10\x07a@\x19V[\x03a\x10kW\x82T`@Qc\x9E\xE4\xAF\xA3`\xE0\x1B\x81R`\x01`\x01`\xA0\x1B\x03\x90\x91\x16\x90c\x9E\xE4\xAF\xA3\x90a\x10=\x90\x89\x90\x89\x90`\x04\x01aI\xDAV[_`@Q\x80\x83\x03\x81_\x87\x80;\x15\x80\x15a\x10TW__\xFD[PZ\xF1\x15\x80\x15a\x10fW=__>=_\xFD[PPPP[PPPa\x10va5\xADV[PPPPPPPPPV[\x7F\xABf\xE1\x1COq,\xD0j\xB1\x1B\xF93\x9BH\xBE\xF3\x9E\x12\xD4\xA2.\xEE\xF7\x1D(`\xA0\xC9\x04\x82\xBDa\x10\xAB\x81a.\x99V[c\xFF\xFF\xFF\xFF\x82\x16\x15\x80a\x10\xC9WP`~Tc\xFF\xFF\xFF\xFF\x90\x81\x16\x90\x83\x16\x11[\x15a\x10\xE7W`@Qcu\x12\xE5\xCB`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[c\xFF\xFF\xFF\xFF\x82\x16_\x90\x81R`\x7F` R`@\x90 `\x01\x81\x01T`\x01`\xE8\x1B\x90\x04`\xFF\x16\x15a\x11(W`@Qc;\x8D=\x99`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x01\x81\x01\x80T`\xFF`\xE8\x1B\x19\x16`\x01`\xE8\x1B\x17\x90U`@Qc\xFF\xFF\xFF\xFF\x84\x16\x90\x7FG\x10\xD2\xEEV~\xF1\xEDn\xB2\xF6Q\xDD\xE4X\x95$\xBC\xF7\xCE\xBCb\x14z\x99\xB2\x81\xCC\x83n~D\x90_\x90\xA2PPPV[a\x11\xD5`@\x80Qa\x01\x80\x81\x01\x82R_\x80\x82R` \x82\x01\x81\x90R\x91\x81\x01\x82\x90R``\x81\x01\x82\x90R`\x80\x81\x01\x82\x90R`\xA0\x81\x01\x82\x90R`\xC0\x81\x01\x82\x90R`\xE0\x81\x01\x82\x90Ra\x01\0\x81\x01\x82\x90R\x90a\x01 \x82\x01\x90\x81R` \x01_\x81R` \x01_\x81RP\x90V[c\xFF\xFF\xFF\xFF\x82\x16_\x90\x81R`\x81` \x90\x81R`@\x91\x82\x90 \x80T`\x01`\x01`\xA0\x1B\x03\x80\x82\x16\x86R`\x01`\x01`@\x1B\x03`\x01`\xA0\x1B\x92\x83\x90\x04\x81\x16\x94\x87\x01\x94\x90\x94R`\x01\x83\x01T\x90\x81\x16\x94\x86\x01\x94\x90\x94R\x90\x92\x04\x81\x16``\x84\x01R`\x05\x82\x01T`\x80\x84\x01R`\x06\x82\x01T\x80\x82\x16`\xA0\x85\x01R`\x01`@\x1B\x90\x81\x90\x04\x82\x16`\xC0\x85\x01R`\x07\x83\x01T\x80\x83\x16`\xE0\x86\x01R\x90\x81\x04\x90\x91\x16a\x01\0\x84\x01Ra\x01 \x83\x01\x90`\xFF`\x01`\x80\x1B\x90\x91\x04\x16`\x02\x81\x11\x15a\x12\x91Wa\x12\x91a@\x19V[\x90\x81`\x02\x81\x11\x15a\x12\xA4Wa\x12\xA4a@\x19V[\x90RP`\x08\x81\x01Ta\x01@\x83\x01R`\t\x01Ta\x01`\x82\x01R\x91\x90PV[c\xFF\xFF\xFF\xFF\x86\x16_\x90\x81R`\x81` R`@\x90 ``\x90a\x12\xE6\x90\x87\x87\x87\x87\x87a5\xD7V[\x97\x96PPPPPPPV[_T`\x04\x90a\x01\0\x90\x04`\xFF\x16\x15\x80\x15a\x13\x11WP_T`\xFF\x80\x83\x16\x91\x16\x10[a\x13\x88W`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`.`$\x82\x01R\x7FInitializable: contract is alrea`D\x82\x01R\x7Fdy initialized\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`d\x82\x01R`\x84\x01[`@Q\x80\x91\x03\x90\xFD[_\x80Ta\xFF\xFF\x19\x16`\xFF\x83\x16\x17a\x01\0\x17\x90U`@\x80Q\x80\x82\x01\x82R`\t\x81Rh\x06\x16\xC2\xD7c\x02\xE32\xE3`\xBC\x1B` \x82\x01R\x90Q\x7FP\xCA\xDC\x0C\0\x1F\x05\xDDK\x81\xDB\x1E\x92\xB9\x8Dw\xE7\x18\xFD/\x10?\xB7\xB7r\x93\xE8g\xD3)\xA4\xC2\x91a\x13\xE7\x91aC\x1AV[`@Q\x80\x91\x03\x90\xA1_\x80Ta\xFF\0\x19\x16\x90U`@Q`\xFF\x82\x16\x81R\x7F\x7F&\xB8?\xF9n\x1F+jh/\x138R\xF6y\x8A\t\xC4e\xDA\x95\x92\x14`\xCE\xFB8G@$\x98\x90` \x01`@Q\x80\x91\x03\x90\xA1PV[3_\x90\x81R\x7F\xF1OZ\x8A\xD5\x9D\x90Y6\x02\xE9\x05\xB3X\"\x9B\xFF\\\xEE\xA6w\xD5\xBF\x0FZ\x17\x01y5P\xA9\xA6` R`@\x90 T`\xFF\x16\x15\x80\x15a\x14\xE1WP3`\x01`\x01`\xA0\x1B\x03\x16\x82`\x01`\x01`\xA0\x1B\x03\x16c\xF8Q\xA4@`@Q\x81c\xFF\xFF\xFF\xFF\x16`\xE0\x1B\x81R`\x04\x01` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a\x14\xB1W=__>=_\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\x14\xD5\x91\x90aI\xEDV[`\x01`\x01`\xA0\x1B\x03\x16\x14\x15[\x15a\x14\xFFW`@Qc\r\x03h\x7F`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x01`\x01`\xA0\x1B\x03\x82\x16_\x90\x81R`\x82` R`@\x81 Tc\xFF\xFF\xFF\xFF\x16\x90\x81\x90\x03a\x15>W`@Qct\xA0\x86\xA3`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[c\xFF\xFF\xFF\xFF\x81\x16_\x90\x81R`\x81` R`@\x81 \x90`\x07\x82\x01T`\x01`\x80\x1B\x90\x04`\xFF\x16`\x02\x81\x11\x15a\x15sWa\x15sa@\x19V[\x14a\x15\x91W`@Qc\x90\xDB\r\x07`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x06\x81\x01T`\x01`\x01`@\x1B\x03\x90\x81\x16\x90\x84\x16\x81\x11\x15\x80a\x15\xC9WP`\x06\x82\x01T`\x01`\x01`@\x1B\x03`\x01`@\x1B\x90\x91\x04\x81\x16\x90\x85\x16\x10[\x15a\x15\xE7W`@Qc\xCB#\xEB\xDF`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x80[\x84`\x01`\x01`@\x1B\x03\x16\x81`\x01`\x01`@\x1B\x03\x16\x14a\x16\x88W`\x01`\x01`@\x1B\x03\x80\x82\x16_\x90\x81R`\x03\x85\x01` R`@\x90 `\x01\x01T`\x01`@\x1B\x90\x04\x81\x16\x90\x86\x16\x81\x10\x15a\x16LW`@Qc\x97S\x96_`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x01`\x01`@\x1B\x03\x90\x91\x16_\x90\x81R`\x03\x84\x01` R`@\x81 \x90\x81U`\x01\x01\x80To\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x19\x16\x90Ua\x15\xE9V[`\x06\x83\x01\x80Tg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x19\x16`\x01`\x01`@\x1B\x03\x87\x16\x17\x90Ua\x16\xB0\x85\x83aJ\x08V[`\x84\x80T_\x90a\x16\xCA\x90\x84\x90`\x01`\x01`@\x1B\x03\x16aJ\x08V[\x82Ta\x01\0\x92\x90\x92\n`\x01`\x01`@\x1B\x03\x81\x81\x02\x19\x90\x93\x16\x91\x83\x16\x02\x17\x90\x91U\x86\x16_\x81\x81R`\x03\x86\x01` R`@\x90\x81\x90 T\x90Qc3Mog`\xE1\x1B\x81R`\x04\x81\x01\x92\x90\x92R`$\x82\x01R`\x01`\x01`\xA0\x1B\x03\x88\x16\x91Pcf\x9A\xDE\xCE\x90`D\x01_`@Q\x80\x83\x03\x81_\x87\x80;\x15\x80\x15a\x17CW__\xFD[PZ\xF1\x15\x80\x15a\x17UW=__>=_\xFD[PPPP`\x01`\x01`@\x1B\x03\x85\x16_\x81\x81R`\x03\x85\x01` \x90\x81R`@\x91\x82\x90 T\x91Q\x91\x82Rc\xFF\xFF\xFF\xFF\x87\x16\x91\x7F\x80\xA6\xD3\x95\xA5Z\xED\x81&\x07\x9C\xB8$\x7F\nhH\xB1D\x0C\xA2\xCD\xCA;C\x86\xF2P\xC3R\x94\x02\x91\x01`@Q\x80\x91\x03\x90\xA3PPPPPPV[`@Qcp\xA0\x821`\xE0\x1B\x81R0`\x04\x82\x01R_\x90\x81\x90`\x01`\x01`\xA0\x1B\x03\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x16\x90cp\xA0\x821\x90`$\x01` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a\x18\x1DW=__>=_\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\x18A\x91\x90aIAV[`\x84T\x90\x91P_\x90a\x18e\x90`\x01`\x01`@\x1B\x03`\x01`@\x1B\x82\x04\x81\x16\x91\x16aJ\x08V[`\x01`\x01`@\x1B\x03\x16\x90P\x80_\x03a\x18\x7FW_\x92PPP\x90V[a\x18\x89\x81\x83aJ;V[\x92PPP\x90V[`oT_\x90`\xFF\x16\x15a\x18\xB6W`@Qc\x0B\xC0\x11\xFF`\xE2\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[3_\x90\x81R`\x82` R`@\x81 Tc\xFF\xFF\xFF\xFF\x16\x90\x81\x90\x03a\x18\xECW`@Qcqe<\x15`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x83`\x01`\x01`@\x1B\x03\x16_\x03a\x19\x15W`@Qc%\x90\xCC\xF9`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[c\xFF\xFF\xFF\xFF\x81\x16_\x90\x81R`\x81` R`@\x81 \x90`\x07\x82\x01T`\x01`\x80\x1B\x90\x04`\xFF\x16`\x02\x81\x11\x15a\x19JWa\x19Ja@\x19V[\x14a\x19hW`@Qc\x90\xDB\r\x07`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x84\x80T\x86\x91\x90_\x90a\x19\x85\x90\x84\x90`\x01`\x01`@\x1B\x03\x16aI\x0BV[\x82Ta\x01\0\x92\x90\x92\n`\x01`\x01`@\x1B\x03\x81\x81\x02\x19\x90\x93\x16\x91\x83\x16\x02\x17\x90\x91U`\x06\x83\x01T\x16\x90P_a\x19\xB8\x87\x83aI\x0BV[`\x06\x84\x01\x80T`\x01`\x01`@\x1B\x03\x83\x81\x16g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x19\x90\x92\x16\x82\x17\x90\x92U`@\x80Q``\x81\x01\x82R\x8A\x81RB\x84\x16` \x80\x83\x01\x91\x82R\x88\x86\x16\x83\x85\x01\x90\x81R_\x86\x81R`\x03\x8C\x01\x83R\x85\x90 \x93Q\x84U\x91Q`\x01\x93\x90\x93\x01\x80T\x92Q\x87\x16`\x01`@\x1B\x02o\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x19\x90\x93\x16\x93\x90\x96\x16\x92\x90\x92\x17\x17\x90\x93UQ\x90\x81R\x91\x92Pc\xFF\xFF\xFF\xFF\x86\x16\x91\x7F\x1D\x9F0&\0Q\xD5\x1Dp3\x9D\xA29\xEA{\x08\0!\xAD\xCA\xAB\xFAq\xC9\xB0\xEA3\x9A \xCF\x9A%\x91\x01`@Q\x80\x91\x03\x90\xA2\x96\x95PPPPPPV[`\x80T_\x90c\xFF\xFF\xFF\xFF\x16\x80\x82\x03a\x1A\x9FWP_\x91\x90PV[_\x81`\x01`\x01`@\x1B\x03\x81\x11\x15a\x1A\xB8Wa\x1A\xB8aC\x97V[`@Q\x90\x80\x82R\x80` \x02` \x01\x82\x01`@R\x80\x15a\x1A\xE1W\x81` \x01` \x82\x02\x806\x837\x01\x90P[P\x90P_[\x82\x81\x10\x15a\x1B>W`\x81_a\x1A\xFC\x83`\x01aJNV[c\xFF\xFF\xFF\xFF\x16c\xFF\xFF\xFF\xFF\x16\x81R` \x01\x90\x81R` \x01_ `\x05\x01T\x82\x82\x81Q\x81\x10a\x1B+Wa\x1B+aJaV[` \x90\x81\x02\x91\x90\x91\x01\x01R`\x01\x01a\x1A\xE6V[P_` [\x83`\x01\x14a\x1DYW_a\x1BW`\x02\x86aJuV[a\x1Bb`\x02\x87aJ;V[a\x1Bl\x91\x90aJNV[\x90P_\x81`\x01`\x01`@\x1B\x03\x81\x11\x15a\x1B\x87Wa\x1B\x87aC\x97V[`@Q\x90\x80\x82R\x80` \x02` \x01\x82\x01`@R\x80\x15a\x1B\xB0W\x81` \x01` \x82\x02\x806\x837\x01\x90P[P\x90P_[\x82\x81\x10\x15a\x1D\tWa\x1B\xC8`\x01\x84aJ\x88V[\x81\x14\x80\x15a\x1B\xE0WPa\x1B\xDC`\x02\x88aJuV[`\x01\x14[\x15a\x1C]W\x85a\x1B\xF1\x82`\x02aI*V[\x81Q\x81\x10a\x1C\x01Wa\x1C\x01aJaV[` \x02` \x01\x01Q\x85`@Q` \x01a\x1C$\x92\x91\x90\x91\x82R` \x82\x01R`@\x01\x90V[`@Q` \x81\x83\x03\x03\x81R\x90`@R\x80Q\x90` \x01 \x82\x82\x81Q\x81\x10a\x1CLWa\x1CLaJaV[` \x02` \x01\x01\x81\x81RPPa\x1D\x01V[\x85a\x1Ci\x82`\x02aI*V[\x81Q\x81\x10a\x1CyWa\x1CyaJaV[` \x02` \x01\x01Q\x86\x82`\x02a\x1C\x8F\x91\x90aI*V[a\x1C\x9A\x90`\x01aJNV[\x81Q\x81\x10a\x1C\xAAWa\x1C\xAAaJaV[` \x02` \x01\x01Q`@Q` \x01a\x1C\xCC\x92\x91\x90\x91\x82R` \x82\x01R`@\x01\x90V[`@Q` \x81\x83\x03\x03\x81R\x90`@R\x80Q\x90` \x01 \x82\x82\x81Q\x81\x10a\x1C\xF4Wa\x1C\xF4aJaV[` \x02` \x01\x01\x81\x81RPP[`\x01\x01a\x1B\xB5V[P\x80\x94P\x81\x95P\x83\x84`@Q` \x01a\x1D,\x92\x91\x90\x91\x82R` \x82\x01R`@\x01\x90V[`@Q` \x81\x83\x03\x03\x81R\x90`@R\x80Q\x90` \x01 \x93P\x82\x80a\x1DO\x90aJ\x9BV[\x93PPPPa\x1BCV[_\x83_\x81Q\x81\x10a\x1DlWa\x1DlaJaV[` \x02` \x01\x01Q\x90P__\x90P[\x82\x81\x10\x15a\x1D\xE2W`@\x80Q` \x81\x01\x84\x90R\x90\x81\x01\x85\x90R``\x01`@\x80Q`\x1F\x19\x81\x84\x03\x01\x81R\x82\x82R\x80Q` \x91\x82\x01 \x90\x83\x01\x87\x90R\x90\x82\x01\x86\x90R\x92P``\x01`@\x80Q`\x1F\x19\x81\x84\x03\x01\x81R\x91\x90R\x80Q` \x90\x91\x01 \x93P`\x01\x01a\x1D{V[P\x95\x94PPPPPV[\x7F\xACu\xD2M\xBB5\xEA\x80\xE2_\xAB\x16}\xA4\xDE\xA4l\x19\x15&\x04&W\r\xB8O\x18H\x91\xF5\xF5\x90a\x1E\x16\x81a.\x99V[`~\x80T_\x91\x90\x82\x90a\x1E.\x90c\xFF\xFF\xFF\xFF\x16aJ\xB0V[\x91\x90a\x01\0\n\x81T\x81c\xFF\xFF\xFF\xFF\x02\x19\x16\x90\x83c\xFF\xFF\xFF\xFF\x16\x02\x17\x90U\x90P`\x01`\x02\x81\x11\x15a\x1E`Wa\x1E`a@\x19V[\x86`\x02\x81\x11\x15a\x1ErWa\x1Era@\x19V[\x03a\x1E\x9BW\x84\x15a\x1E\x96W`@Qcc\xD7\"\xE7`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\x1FUV[`\x02\x86`\x02\x81\x11\x15a\x1E\xAFWa\x1E\xAFa@\x19V[\x03a\x1F\x05W`\x01`\x01`\xA0\x1B\x03\x88\x16\x15\x15\x80a\x1E\xD3WP`\x01`\x01`@\x1B\x03\x87\x16\x15\x15[\x80a\x1E\xDDWP\x84\x15\x15[\x80a\x1E\xE7WP\x82\x15\x15[\x15a\x1E\x96W`@Qcc\xD7\"\xE7`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[_\x86`\x02\x81\x11\x15a\x1F\x18Wa\x1F\x18a@\x19V[\x03a\x1F<W\x82\x15a\x1E\x96W`@Qcc\xD7\"\xE7`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`@Qcc\xD7\"\xE7`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`@Q\x80`\xE0\x01`@R\x80\x8A`\x01`\x01`\xA0\x1B\x03\x16\x81R` \x01\x89`\x01`\x01`\xA0\x1B\x03\x16\x81R` \x01\x88`\x01`\x01`@\x1B\x03\x16\x81R` \x01\x87`\x02\x81\x11\x15a\x1F\x9FWa\x1F\x9Fa@\x19V[\x81R_` \x80\x83\x01\x82\x90R`@\x80\x84\x01\x8A\x90R``\x93\x84\x01\x88\x90Rc\xFF\xFF\xFF\xFF\x86\x16\x83R`\x7F\x82R\x91\x82\x90 \x84Q\x81T`\x01`\x01`\xA0\x1B\x03\x91\x82\x16s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x19\x90\x91\x16\x17\x82U\x91\x85\x01Q`\x01\x82\x01\x80T\x94\x87\x01Q`\x01`\x01`@\x1B\x03\x16`\x01`\xA0\x1B\x02`\x01`\x01`\xE0\x1B\x03\x19\x90\x95\x16\x91\x90\x93\x16\x17\x92\x90\x92\x17\x80\x82U\x92\x84\x01Q\x91\x92`\xFF`\xE0\x1B\x19\x16`\x01`\xE0\x1B\x83`\x02\x81\x11\x15a OWa Oa@\x19V[\x02\x17\x90UP`\x80\x82\x01Q`\x01\x82\x01\x80T\x91\x15\x15`\x01`\xE8\x1B\x02`\xFF`\xE8\x1B\x19\x90\x92\x16\x91\x90\x91\x17\x90U`\xA0\x82\x01Q`\x02\x82\x01U`\xC0\x90\x91\x01Q`\x03\x90\x91\x01U`@Qc\xFF\xFF\xFF\xFF\x82\x16\x90\x7F\x9E\xAF.\xCB\xDD\xB1H\x89\xC9\xE1A\xA61u\xC5Z\xC2^\x0C\xD7\xCD\xEA1,\xDF\xBD\x03\x97\x97k8:\x90a \xD1\x90\x8C\x90\x8C\x90\x8C\x90\x8C\x90\x8C\x90\x8C\x90\x8C\x90aJ\xD4V[`@Q\x80\x91\x03\x90\xA2PPPPPPPPPV[\x7Ff\x15f\x03\xFE)\xD1?\x97\xC6\xF3\xE3\xDF\xF4\xEFq\x91\x9F\x9A\xA6\x1CU[\xE0\x18-\x95N\x94\"\x1A\xACa!\x0E\x81a.\x99V[a!\x19\x84\x84\x84a6\xDAV[PPPPV[\x7F\xA0\xFA\xB0t\xAB\xA3jo\xA6\x9F\x1A\x83\xEE\x86\xE5\xAB\xFB\x843\x96n\xB5~\xFB\x13\xDC/\xC2\xF2M\xDD\x08a!I\x81a.\x99V[c\xFF\xFF\xFF\xFF\x89\x16\x15\x80a!gWP`~Tc\xFF\xFF\xFF\xFF\x90\x81\x16\x90\x8A\x16\x11[\x15a!\x85W`@Qcu\x12\xE5\xCB`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[c\xFF\xFF\xFF\xFF\x89\x16_\x90\x81R`\x7F` R`@\x90 `\x01\x81\x01T`\x01`\xE8\x1B\x90\x04`\xFF\x16\x15a!\xC6W`@Qc;\x8D=\x99`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[c\xFF\xFF\xFF\xFF`\x01`\x01`@\x1B\x03\x8A\x16\x11\x15a!\xF4W`@QcLu?W`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x01`\x01`@\x1B\x03\x89\x16_\x90\x81R`\x83` R`@\x90 Tc\xFF\xFF\xFF\xFF\x16\x15a\"0W`@Qc7\xC8\xFE\t`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x80\x80T_\x91\x90\x82\x90a\"H\x90c\xFF\xFF\xFF\xFF\x16aJ\xB0V[\x82Tc\xFF\xFF\xFF\xFF\x82\x81\x16a\x01\0\x94\x90\x94\n\x93\x84\x02\x93\x02\x19\x16\x91\x90\x91\x17\x90\x91U\x82T`@\x80Q_\x80\x82R` \x82\x01\x92\x83\x90R\x93\x94P`\x01`\x01`\xA0\x1B\x03\x90\x92\x16\x910\x91a\"\x93\x90a>\xA0V[a\"\x9F\x93\x92\x91\x90aK:V[`@Q\x80\x91\x03\x90_\xF0\x80\x15\x80\x15a\"\xB8W=__>=_\xFD[P\x90P\x81`\x83_\x8D`\x01`\x01`@\x1B\x03\x16`\x01`\x01`@\x1B\x03\x16\x81R` \x01\x90\x81R` \x01_ _a\x01\0\n\x81T\x81c\xFF\xFF\xFF\xFF\x02\x19\x16\x90\x83c\xFF\xFF\xFF\xFF\x16\x02\x17\x90UP\x81`\x82_\x83`\x01`\x01`\xA0\x1B\x03\x16`\x01`\x01`\xA0\x1B\x03\x16\x81R` \x01\x90\x81R` \x01_ _a\x01\0\n\x81T\x81c\xFF\xFF\xFF\xFF\x02\x19\x16\x90\x83c\xFF\xFF\xFF\xFF\x16\x02\x17\x90UP_`\x81_\x84c\xFF\xFF\xFF\xFF\x16c\xFF\xFF\xFF\xFF\x16\x81R` \x01\x90\x81R` \x01_ \x90P\x81\x81_\x01_a\x01\0\n\x81T\x81`\x01`\x01`\xA0\x1B\x03\x02\x19\x16\x90\x83`\x01`\x01`\xA0\x1B\x03\x16\x02\x17\x90UP\x83`\x01\x01`\x14\x90T\x90a\x01\0\n\x90\x04`\x01`\x01`@\x1B\x03\x16\x81`\x01\x01`\x14a\x01\0\n\x81T\x81`\x01`\x01`@\x1B\x03\x02\x19\x16\x90\x83`\x01`\x01`@\x1B\x03\x16\x02\x17\x90UP\x83`\x01\x01_\x90T\x90a\x01\0\n\x90\x04`\x01`\x01`\xA0\x1B\x03\x16\x81`\x01\x01_a\x01\0\n\x81T\x81`\x01`\x01`\xA0\x1B\x03\x02\x19\x16\x90\x83`\x01`\x01`\xA0\x1B\x03\x16\x02\x17\x90UP\x8B\x81_\x01`\x14a\x01\0\n\x81T\x81`\x01`\x01`@\x1B\x03\x02\x19\x16\x90\x83`\x01`\x01`@\x1B\x03\x16\x02\x17\x90UP\x83`\x02\x01T\x81`\x02\x01__`\x01`\x01`@\x1B\x03\x16\x81R` \x01\x90\x81R` \x01_ \x81\x90UP\x8Cc\xFF\xFF\xFF\xFF\x16\x81`\x07\x01`\x08a\x01\0\n\x81T\x81`\x01`\x01`@\x1B\x03\x02\x19\x16\x90\x83`\x01`\x01`@\x1B\x03\x16\x02\x17\x90UP\x83`\x01\x01`\x1C\x90T\x90a\x01\0\n\x90\x04`\xFF\x16\x81`\x07\x01`\x10a\x01\0\n\x81T\x81`\xFF\x02\x19\x16\x90\x83`\x02\x81\x11\x15a$\xB5Wa$\xB5a@\x19V[\x02\x17\x90UP\x83`\x03\x01T\x81`\t\x01\x81\x90UP\x82c\xFF\xFF\xFF\xFF\x16\x7F\x19L\x984V\xDFg\x01\xC6\xA5\x080\xB9\x0F\xE8\x0Er\xB8#A\x1D\rRIp\xC9Y\r\xC2w\xA6A\x8E\x84\x8F\x8D`@Qa%6\x94\x93\x92\x91\x90c\xFF\xFF\xFF\xFF\x94\x90\x94\x16\x84R`\x01`\x01`\xA0\x1B\x03\x92\x83\x16` \x85\x01R`\x01`\x01`@\x1B\x03\x91\x90\x91\x16`@\x84\x01R\x16``\x82\x01R`\x80\x01\x90V[`@Q\x80\x91\x03\x90\xA2`\x02`\x01\x85\x01T`\x01`\xE0\x1B\x90\x04`\xFF\x16`\x02\x81\x11\x15a%`Wa%`a@\x19V[\x03a%\xC3W`@QcC\x9F\xAB\x91`\xE0\x1B\x81R`\x01`\x01`\xA0\x1B\x03\x83\x16\x90cC\x9F\xAB\x91\x90a%\x91\x90\x89\x90`\x04\x01aC\x1AV[_`@Q\x80\x83\x03\x81_\x87\x80;\x15\x80\x15a%\xA8W__\xFD[PZ\xF1\x15\x80\x15a%\xBAW=__>=_\xFD[PPPPa&'V[`@Qc8\x92\xB8\x11`\xE1\x1B\x81R`\x01`\x01`\xA0\x1B\x03\x83\x16\x90cq%p\"\x90a%\xF9\x90\x8E\x90\x8E\x90\x88\x90\x8F\x90\x8F\x90\x8F\x90`\x04\x01aKsV[_`@Q\x80\x83\x03\x81_\x87\x80;\x15\x80\x15a&\x10W__\xFD[PZ\xF1\x15\x80\x15a&\"W=__>=_\xFD[PPPP[PPPPPPPPPPPPPV[\x7F\x8C\xF8\x07\xF6\x97\x07 \xF8\xE2\xC2\x08\xC7\xC5\x03u\x95\x98,{\xD9\xED\x93\xC3\x80\xD0\x9D\xF7C\xD0\xDC\xC3\xFBa&`\x81a.\x99V[h65\xC9\xAD\xC5\xDE\xA0\0\0\x82\x11\x80a&zWPc;\x9A\xCA\0\x82\x10[\x15a&\x98W`@Qc\x85\x86\x95%`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x86\x82\x90U`@Q\x82\x81R\x7F\xFB86S\xF5>\xE0y\x97\x8D\x0C\x9A\xFFz\xEF\xF0J\x10\x16l\xE2D\xCC\xA9\xC9\xF9\xD8\xD9k\xEDE\xB2\x90` \x01`@Q\x80\x91\x03\x90\xA1PPV[_\x82\x81R`4` R`@\x90 `\x01\x01Ta&\xEE\x81a.\x99V[a\x0C@\x83\x83a2\xEAV[\x7Fb\xBAk\xA2\xFF\xED\x8C\xFE1kX3%\xEAA\xACn{\xA9\xE5\x86M+\xC6\xFA\xBB\xA7\xAC&\xD2\xF0\xF4a'\"\x81a.\x99V[`\x87\x80Tg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x19\x16B`\x01`\x01`@\x1B\x03\x16\x17\x90U`@\x80Qcm\xE0\xB4\xBB`\xE1\x1B\x81R\x90Q\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`\x01`\x01`\xA0\x1B\x03\x16\x91c\xDB\xC1iv\x91`\x04\x80\x83\x01\x92_\x92\x91\x90\x82\x90\x03\x01\x81\x83\x87\x80;\x15\x80\x15a'\x9DW__\xFD[PZ\xF1\x15\x80\x15a'\xAFW=__>=_\xFD[PPPPa'\xBBa:nV[PV[c\xFF\xFF\xFF\xFF\x86\x16_\x90\x81R`\x81` R`@\x90 ``\x90a\x12\xE6\x90\x88\x90\x88\x88\x88\x88\x88a3\xD8V[3`\x01`\x01`\xA0\x1B\x03\x16\x82`\x01`\x01`\xA0\x1B\x03\x16c\xF8Q\xA4@`@Q\x81c\xFF\xFF\xFF\xFF\x16`\xE0\x1B\x81R`\x04\x01` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a(+W=__>=_\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a(O\x91\x90aI\xEDV[`\x01`\x01`\xA0\x1B\x03\x16\x14a(vW`@Qci`r\xE9`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x01`\x01`\xA0\x1B\x03\x82\x16_\x90\x81R`\x82` \x90\x81R`@\x80\x83 Tc\xFF\xFF\xFF\xFF\x16\x83R`\x81\x90\x91R\x90 `\x06\x81\x01T`\x01`\x01`@\x1B\x03\x80\x82\x16`\x01`@\x1B\x90\x92\x04\x16\x14a(\xD7W`@QcfC\x16\xA5`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x02c\xFF\xFF\xFF\xFF\x83\x16_\x90\x81R`\x7F` R`@\x90 `\x01\x01T`\x01`\xE0\x1B\x90\x04`\xFF\x16`\x02\x81\x11\x15a)\x0CWa)\x0Ca@\x19V[\x14\x15\x80\x15a)5WP`\x07\x81\x01Tc\xFF\xFF\xFF\xFF\x83\x16`\x01`@\x1B\x90\x91\x04`\x01`\x01`@\x1B\x03\x16\x10\x15[\x15a)SW`@Qc>7\xE23`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x01`\x01`\xA0\x1B\x03\x83\x16_\x90\x81R`\x82` \x90\x81R`@\x80\x83 Tc\xFF\xFF\xFF\xFF\x86\x81\x16\x85R`\x7F\x90\x93R\x92 `\x01\x01T\x91\x16\x90`\x01`\xE0\x1B\x90\x04`\xFF\x16`\x02\x81\x11\x15a)\xA1Wa)\xA1a@\x19V[c\xFF\xFF\xFF\xFF\x82\x16_\x90\x81R`\x7F` R`@\x90 `\x01\x01T`\x01`\xE0\x1B\x90\x04`\xFF\x16`\x02\x81\x11\x15a)\xD4Wa)\xD4a@\x19V[\x14a)\xF2W`@QcZ\xA0\xD5\xF1`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`@\x80Q_\x81R` \x81\x01\x90\x91Ra!\x19\x90\x85\x90\x85\x90a6\xDAV[\x7F=\xFE'}*,\x04\xB7_\xB2\xEB7C\xFA\0\0Z\xE3g\x8A \xC2\x99\xE6_\xDFM\xF7e\x17\xF6\x8Ea*7\x81a.\x99V[`\x01`\x01`@\x1B\x03\x86\x16_\x90\x81R`\x83` R`@\x90 Tc\xFF\xFF\xFF\xFF\x16\x15a*sW`@Qc7\xC8\xFE\t`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[c\xFF\xFF\xFF\xFF`\x01`\x01`@\x1B\x03\x87\x16\x11\x15a*\xA1W`@QcLu?W`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x01`\x01`\xA0\x1B\x03\x89\x16_\x90\x81R`\x82` R`@\x90 Tc\xFF\xFF\xFF\xFF\x16\x15a*\xDDW`@Qc\r@\x9B\x93`\xE4\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x80\x80T_\x91\x90\x82\x90a*\xF5\x90c\xFF\xFF\xFF\xFF\x16aJ\xB0V[\x91\x90a\x01\0\n\x81T\x81c\xFF\xFF\xFF\xFF\x02\x19\x16\x90\x83c\xFF\xFF\xFF\xFF\x16\x02\x17\x90U\x90P\x80`\x83_\x89`\x01`\x01`@\x1B\x03\x16`\x01`\x01`@\x1B\x03\x16\x81R` \x01\x90\x81R` \x01_ _a\x01\0\n\x81T\x81c\xFF\xFF\xFF\xFF\x02\x19\x16\x90\x83c\xFF\xFF\xFF\xFF\x16\x02\x17\x90UP\x80`\x82_\x8C`\x01`\x01`\xA0\x1B\x03\x16`\x01`\x01`\xA0\x1B\x03\x16\x81R` \x01\x90\x81R` \x01_ _a\x01\0\n\x81T\x81c\xFF\xFF\xFF\xFF\x02\x19\x16\x90\x83c\xFF\xFF\xFF\xFF\x16\x02\x17\x90UP_`\x81_\x83c\xFF\xFF\xFF\xFF\x16c\xFF\xFF\xFF\xFF\x16\x81R` \x01\x90\x81R` \x01_ \x90P\x8A\x81_\x01_a\x01\0\n\x81T\x81`\x01`\x01`\xA0\x1B\x03\x02\x19\x16\x90\x83`\x01`\x01`\xA0\x1B\x03\x16\x02\x17\x90UP\x88\x81`\x01\x01`\x14a\x01\0\n\x81T\x81`\x01`\x01`@\x1B\x03\x02\x19\x16\x90\x83`\x01`\x01`@\x1B\x03\x16\x02\x17\x90UP\x89\x81`\x01\x01_a\x01\0\n\x81T\x81`\x01`\x01`\xA0\x1B\x03\x02\x19\x16\x90\x83`\x01`\x01`\xA0\x1B\x03\x16\x02\x17\x90UP\x87\x81_\x01`\x14a\x01\0\n\x81T\x81`\x01`\x01`@\x1B\x03\x02\x19\x16\x90\x83`\x01`\x01`@\x1B\x03\x16\x02\x17\x90UP\x85\x81`\x07\x01`\x10a\x01\0\n\x81T\x81`\xFF\x02\x19\x16\x90\x83`\x02\x81\x11\x15a,|Wa,|a@\x19V[\x02\x17\x90UP`\x01\x86`\x02\x81\x11\x15a,\x95Wa,\x95a@\x19V[\x03a,\xADW`\t\x81\x01\x85\x90U`\x05\x81\x01\x87\x90Ua,\xECV[`\x02\x86`\x02\x81\x11\x15a,\xC1Wa,\xC1a@\x19V[\x03a,\xD9W`\x08\x81\x01\x84\x90U`\x05\x81\x01\x87\x90Ua,\xECV[_\x80\x80R`\x02\x82\x01` R`@\x90 \x87\x90U[\x81c\xFF\xFF\xFF\xFF\x16\x7FM\xA4\x7Fn\x9B\xBD\x9E\xF9\x18\x87\x18:Wj\xAE\xBC\xF1\xB9\xBB}*V{3\xB0u\x04Lm6\x08.\x8A\x8D\x8B\x8A_\x8B\x8B`@Qa-.\x97\x96\x95\x94\x93\x92\x91\x90aK\xDDV[`@Q\x80\x91\x03\x90\xA2PPPPPPPPPPPV[a-\xA7`@\x80Qa\x01\x80\x81\x01\x82R_\x80\x82R` \x82\x01\x81\x90R\x91\x81\x01\x82\x90R``\x81\x01\x82\x90R`\x80\x81\x01\x82\x90R`\xA0\x81\x01\x82\x90R`\xC0\x81\x01\x82\x90R`\xE0\x81\x01\x82\x90Ra\x01\0\x81\x01\x82\x90Ra\x01 \x81\x01\x82\x90Ra\x01@\x81\x01\x82\x90R\x90a\x01`\x82\x01R\x90V[c\xFF\xFF\xFF\xFF\x82\x16_\x90\x81R`\x81` \x90\x81R`@\x91\x82\x90 \x80T`\x01`\x01`\xA0\x1B\x03\x80\x82\x16\x86R`\x01`\xA0\x1B\x91\x82\x90\x04`\x01`\x01`@\x1B\x03\x90\x81\x16\x94\x87\x01\x94\x90\x94R`\x01\x83\x01T\x90\x81\x16\x94\x86\x01\x94\x90\x94R\x90\x92\x04\x81\x16``\x84\x01R`\x05\x82\x01T`\x80\x84\x01R`\x06\x82\x01T\x80\x82\x16`\xA0\x85\x01R`\x01`@\x1B\x80\x82\x04\x83\x16`\xC0\x86\x01R`\x01`\x80\x1B\x80\x83\x04\x84\x16`\xE0\x87\x01R`\x01`\xC0\x1B\x90\x92\x04\x83\x16a\x01\0\x86\x01R`\x07\x84\x01T\x80\x84\x16a\x01 \x87\x01R\x90\x81\x04\x90\x92\x16a\x01@\x85\x01Ra\x01`\x84\x01\x91\x04`\xFF\x16`\x02\x81\x11\x15a.|Wa.|a@\x19V[\x90\x81`\x02\x81\x11\x15a.\x8FWa.\x8Fa@\x19V[\x81RPPP\x91\x90PV[a'\xBB\x813a:\xC5V[__a.\xC1\x89`\x06\x01T`\x01`\x01`@\x1B\x03`\x01`@\x1B\x90\x91\x04\x16\x90V[`\x07\x8A\x01T\x90\x91P`\x01`\x01`@\x1B\x03\x90\x81\x16\x90\x89\x16\x10\x15a.\xF6W`@Qc\xEA\xD14\x0B`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x01`\x01`@\x1B\x03\x88\x16_\x90\x81R`\x02\x8A\x01` R`@\x90 T\x91P\x81a/0W`@Qc$\xCB\xDC\xC3`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x80`\x01`\x01`@\x1B\x03\x16\x88`\x01`\x01`@\x1B\x03\x16\x11\x15a/cW`@Qc\x0F+t\xF1`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x80`\x01`\x01`@\x1B\x03\x16\x87`\x01`\x01`@\x1B\x03\x16\x11a/\x95W`@Qc\xB9\xB1\x8FW`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[_a/\xA4\x8A\x8A\x8A\x8A\x87\x8Ba5\xD7V[\x90P_\x7F0dNr\xE11\xA0)\xB8PE\xB6\x81\x81X](3\xE8Hy\xB9p\x91C\xE1\xF5\x93\xF0\0\0\x01`\x02\x83`@Qa/\xD8\x91\x90aL8V[` `@Q\x80\x83\x03\x81\x85Z\xFA\x15\x80\x15a/\xF3W=__>=_\xFD[PPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a0\x16\x91\x90aIAV[a0 \x91\x90aJuV[`\x01\x8C\x01T`@\x80Q` \x81\x01\x82R\x83\x81R\x90QcH\x90\xEDE`\xE1\x1B\x81R\x92\x93P`\x01`\x01`\xA0\x1B\x03\x90\x91\x16\x91c\x91!\xDA\x8A\x91a0b\x91\x89\x91\x90`\x04\x01aLNV[` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a0}W=__>=_\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a0\xA1\x91\x90aL\x8AV[a0\xBEW`@Qc\t\xBD\xE39`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[_a0\xC9\x84\x8BaJ\x08V[\x90Pa1\x1C\x87\x82`\x01`\x01`@\x1B\x03\x16a0\xE1a\x17\xB7V[a0\xEB\x91\x90aI*V[`\x01`\x01`\xA0\x1B\x03\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x16\x91\x90a;\x07V[\x80`\x84`\x08\x82\x82\x82\x90T\x90a\x01\0\n\x90\x04`\x01`\x01`@\x1B\x03\x16a1@\x91\x90aI\x0BV[\x82Ta\x01\0\x92\x90\x92\n`\x01`\x01`@\x1B\x03\x81\x81\x02\x19\x90\x93\x16\x91\x83\x16\x02\x17\x90\x91U`\x84\x80Tg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF`\x80\x1B\x19\x16`\x01`\x80\x1BB\x84\x16\x02\x17\x90U\x8DT`@Qc2\xC2\xD1S`\xE0\x1B\x81R\x91\x8D\x16`\x04\x83\x01R`$\x82\x01\x8B\x90R3`D\x83\x01R`\x01`\x01`\xA0\x1B\x03\x16\x91Pc2\xC2\xD1S\x90`d\x01_`@Q\x80\x83\x03\x81_\x87\x80;\x15\x80\x15a1\xCDW__\xFD[PZ\xF1\x15\x80\x15a1\xDFW=__>=_\xFD[PPPPPPPPPPPPPPPPV[\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`\x01`\x01`\xA0\x1B\x03\x16c r\xF6\xC5`@Q\x81c\xFF\xFF\xFF\xFF\x16`\xE0\x1B\x81R`\x04\x01_`@Q\x80\x83\x03\x81_\x87\x80;\x15\x80\x15a2IW__\xFD[PZ\xF1\x15\x80\x15a2[W=__>=_\xFD[PPPPa\x0C\x1Aa;nV[_\x82\x81R`4` \x90\x81R`@\x80\x83 `\x01`\x01`\xA0\x1B\x03\x85\x16\x84R\x90\x91R\x90 T`\xFF\x16a\x0CxW_\x82\x81R`4` \x90\x81R`@\x80\x83 `\x01`\x01`\xA0\x1B\x03\x85\x16\x80\x85R\x92R\x80\x83 \x80T`\xFF\x19\x16`\x01\x17\x90UQ3\x92\x85\x91\x7F/\x87\x88\x11~~\xFF\x1D\x82\xE9&\xECyI\x01\xD1|x\x02JP'\t@0E@\xA73eo\r\x91\x90\xA4PPV[_\x82\x81R`4` \x90\x81R`@\x80\x83 `\x01`\x01`\xA0\x1B\x03\x85\x16\x84R\x90\x91R\x90 T`\xFF\x16\x15a\x0CxW_\x82\x81R`4` \x90\x81R`@\x80\x83 `\x01`\x01`\xA0\x1B\x03\x85\x16\x80\x85R\x92R\x80\x83 \x80T`\xFF\x19\x16\x90UQ3\x92\x85\x91\x7F\xF69\x1F\\2\xD9\xC6\x9D*G\xEAg\x0BD)t\xB595\xD1\xED\xC7\xFDd\xEB!\xE0G\xA89\x17\x1B\x91\x90\xA4PPV[\x7F\x9Bw\x9B\x17B-\r\xF9\"#\x01\x8B2\xB4\xD1\xFAF\xE0qr=h\x17\xE2Hm\0;\xEC\xC5_\0\\\x15a3\xABW`@Qc>\xE5\xAE\xB5`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\x0C\x1A`\x01\x7F\x9Bw\x9B\x17B-\r\xF9\"#\x01\x8B2\xB4\xD1\xFAF\xE0qr=h\x17\xE2Hm\0;\xEC\xC5_\0[\x90a;\xC9V[```\x02`\x07\x88\x01T`\x01`\x80\x1B\x90\x04`\xFF\x16`\x02\x81\x11\x15a3\xFCWa3\xFCa@\x19V[\x03a4\xD7W\x86T`@Qc\x1A\x95}\x9B`\xE2\x1B\x81R_\x91`\x01`\x01`\xA0\x1B\x03\x16\x90cjU\xF6l\x90a42\x90\x87\x90\x87\x90`\x04\x01aI\xDAV[` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a4MW=__>=_\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a4q\x91\x90aIAV[`\x05\x89\x01T`\x08\x8A\x01T`@\x80Q` \x81\x01\x93\x90\x93R\x82\x01R``\x81\x01\x89\x90R`\x01`\x01`\xE0\x1B\x03\x19`\xE0\x8C\x90\x1B\x16`\x80\x82\x01R`\x84\x81\x01\x82\x90R`\xA4\x81\x01\x88\x90R`\xC4\x81\x01\x87\x90R\x90\x91P`\xE4\x01`@Q` \x81\x83\x03\x03\x81R\x90`@R\x91PPa\x12\xE6V[\x86T`@\x80Qc+G\xB7\xCD`\xE2\x1B\x81R\x90Q_\x92`\x01`\x01`\xA0\x1B\x03\x16\x91c\xAD\x1E\xDF4\x91`\x04\x80\x83\x01\x92` \x92\x91\x90\x82\x90\x03\x01\x81\x86Z\xFA\x15\x80\x15a5\x1DW=__>=_\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a5A\x91\x90aIAV[`\x05\x89\x01T`\x08\x8A\x01T`@\x80Q` \x81\x01\x93\x90\x93R\x82\x01R``\x81\x01\x89\x90R`\x01`\x01`\xE0\x1B\x03\x19`\xE0\x8C\x90\x1B\x16`\x80\x82\x01R`\x84\x81\x01\x82\x90R`\xA4\x81\x01\x88\x90R`\xC4\x81\x01\x87\x90R\x90\x91P`\xE4\x01`@Q` \x81\x83\x03\x03\x81R\x90`@R\x91PP\x97\x96PPPPPPPV[a\x0C\x1A_\x7F\x9Bw\x9B\x17B-\r\xF9\"#\x01\x8B2\xB4\xD1\xFAF\xE0qr=h\x17\xE2Hm\0;\xEC\xC5_\0a3\xD2V[`\x01`\x01`@\x1B\x03\x80\x86\x16_\x81\x81R`\x03\x89\x01` R`@\x80\x82 T\x93\x88\x16\x82R\x90 T``\x92\x91\x15\x80\x15\x90a6\x0BWP\x81\x15[\x15a6)W`@Qc4\x0CaO`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x80a6GW`@Qcf8[Q`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a6P\x84a;\xD0V[a6mW`@Qc\x05\xDA\xE4O`\xE2\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[3\x85\x83\x8A\x8C_\x01`\x14\x90T\x90a\x01\0\n\x90\x04`\x01`\x01`@\x1B\x03\x16\x8D`\x01\x01`\x14\x90T\x90a\x01\0\n\x90\x04`\x01`\x01`@\x1B\x03\x16\x89\x87\x8D\x8F`@Q` \x01a6\xBD\x9A\x99\x98\x97\x96\x95\x94\x93\x92\x91\x90aL\xA9V[`@Q` \x81\x83\x03\x03\x81R\x90`@R\x92PPP\x96\x95PPPPPPV[c\xFF\xFF\xFF\xFF\x82\x16\x15\x80a6\xF8WP`~Tc\xFF\xFF\xFF\xFF\x90\x81\x16\x90\x83\x16\x11[\x15a7\x16W`@Qcu\x12\xE5\xCB`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x01`\x01`\xA0\x1B\x03\x83\x16_\x90\x81R`\x82` R`@\x81 Tc\xFF\xFF\xFF\xFF\x16\x90\x81\x90\x03a7UW`@Qct\xA0\x86\xA3`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[c\xFF\xFF\xFF\xFF\x81\x81\x16_\x90\x81R`\x81` R`@\x90 `\x07\x81\x01T\x90\x91\x85\x16`\x01`@\x1B\x90\x91\x04`\x01`\x01`@\x1B\x03\x16\x03a7\xA2W`@QcOa\xD5\x19`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[c\xFF\xFF\xFF\xFF\x84\x16_\x90\x81R`\x7F` R`@\x90 `\x01\x81\x01T`\x01`\xE8\x1B\x90\x04`\xFF\x16\x15a7\xE3W`@Qc;\x8D=\x99`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x02`\x01\x82\x01T`\x01`\xE0\x1B\x90\x04`\xFF\x16`\x02\x81\x11\x15a8\x05Wa8\x05a@\x19V[\x14a8\xA8W`\x02`\x07\x83\x01T`\x01`\x80\x1B\x90\x04`\xFF\x16`\x02\x81\x11\x15a8,Wa8,a@\x19V[\x03a8JW`@QcZ\xA0\xD5\xF1`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x01\x81\x01T`\x01`\xE0\x1B\x90\x04`\xFF\x16`\x02\x81\x11\x15a8jWa8ja@\x19V[`\x07\x83\x01T`\x01`\x80\x1B\x90\x04`\xFF\x16`\x02\x81\x11\x15a8\x8AWa8\x8Aa@\x19V[\x14a8\xA8W`@QcZ\xA0\xD5\xF1`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x01\x80\x82\x01\x80T\x91\x84\x01\x80Ts\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x19\x81\x16`\x01`\x01`\xA0\x1B\x03\x90\x94\x16\x93\x84\x17\x82U\x82T`\x01`\x01`@\x1B\x03`\x01`\xA0\x1B\x91\x82\x90\x04\x16\x02`\x01`\x01`\xE0\x1B\x03\x19\x90\x91\x16\x90\x93\x17\x92\x90\x92\x17\x90\x91U`\x03\x82\x01T`\t\x84\x01U`\x07\x83\x01\x80T`\x01`@\x1Bc\xFF\xFF\xFF\xFF\x89\x16\x02o\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\x19\x82\x16\x81\x17\x83U\x92T`\xFF`\x01`\xE0\x1B\x90\x91\x04\x16\x92p\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x19\x16p\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\x19\x90\x91\x16\x17`\x01`\x80\x1B\x83`\x02\x81\x11\x15a9\x8CWa9\x8Ca@\x19V[\x02\x17\x90UP_a9\x9B\x84a\t:V[`\x07\x84\x01\x80Tg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x19\x16`\x01`\x01`@\x1B\x03\x83\x16\x17\x90U\x82T`@Qc'\x8FyC`\xE1\x1B\x81R\x91\x92P`\x01`\x01`\xA0\x1B\x03\x89\x81\x16\x92cO\x1E\xF2\x86\x92a9\xED\x92\x16\x90\x89\x90`\x04\x01aM]V[_`@Q\x80\x83\x03\x81_\x87\x80;\x15\x80\x15a:\x04W__\xFD[PZ\xF1\x15\x80\x15a:\x16W=__>=_\xFD[PP`@\x80Qc\xFF\xFF\xFF\xFF\x8A\x81\x16\x82R`\x01`\x01`@\x1B\x03\x86\x16` \x83\x01R\x88\x16\x93P\x7F\xF5\x85\xE0L\x05\xD3\x96\x90\x11p$w\x83\xD3\xE5\xF0\xEE\x9C\x1D\xF20r\x98[P\xAF\x08\x9F^H\xB1\x9D\x92P\x01`@Q\x80\x91\x03\x90\xA2PPPPPPPV[`oT`\xFF\x16a:\x91W`@QcS\x86i\x81`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`o\x80T`\xFF\x19\x16\x90U`@Q\x7F\x1E^4\xEE\xA35\x01\xAE\xCF.\xBE\xC9\xFE\x0E\x88J@\x80Bu\xEA\x7F\xE1\x0B+\xA0\x84\xC87C\x08\xB3\x90_\x90\xA1V[_\x82\x81R`4` \x90\x81R`@\x80\x83 `\x01`\x01`\xA0\x1B\x03\x85\x16\x84R\x90\x91R\x90 T`\xFF\x16a\x0CxW`@Qcv\x15\xBE\x1F`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`@\x80Q`\x01`\x01`\xA0\x1B\x03\x84\x16`$\x82\x01R`D\x80\x82\x01\x84\x90R\x82Q\x80\x83\x03\x90\x91\x01\x81R`d\x90\x91\x01\x90\x91R` \x81\x01\x80Q{\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x16c\xA9\x05\x9C\xBB`\xE0\x1B\x17\x90Ra\x0C@\x90\x84\x90a<UV[`oT`\xFF\x16\x15a;\x92W`@Qc\x0B\xC0\x11\xFF`\xE2\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`o\x80T`\xFF\x19\x16`\x01\x17\x90U`@Q\x7F\"a\xEF\xE5\xAE\xF6\xFE\xDC\x1F\xD1U\x0B%\xFA\xCC\x91\x81tV#\x04\x9Cy\x01(p0\xB9\xAD\x1AT\x97\x90_\x90\xA1V[\x80\x82]PPV[_g\xFF\xFF\xFF\xFF\0\0\0\x01`\x01`\x01`@\x1B\x03\x83\x16\x10\x80\x15a<\x05WPg\xFF\xFF\xFF\xFF\0\0\0\x01`@\x83\x90\x1C`\x01`\x01`@\x1B\x03\x16\x10[\x80\x15a<%WPg\xFF\xFF\xFF\xFF\0\0\0\x01`\x80\x83\x90\x1C`\x01`\x01`@\x1B\x03\x16\x10[\x80\x15a<<WPg\xFF\xFF\xFF\xFF\0\0\0\x01`\xC0\x83\x90\x1C\x10[\x15a<IWP`\x01\x91\x90PV[P_\x91\x90PV[\x91\x90PV[_a<\xA9\x82`@Q\x80`@\x01`@R\x80` \x81R` \x01\x7FSafeERC20: low-level call failed\x81RP\x85`\x01`\x01`\xA0\x1B\x03\x16a=&\x90\x92\x91\x90c\xFF\xFF\xFF\xFF\x16V[\x80Q\x90\x91P\x15a\x0C@W\x80\x80` \x01\x90Q\x81\x01\x90a<\xC7\x91\x90aL\x8AV[a\x0C@W`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`*`$\x82\x01R\x7FSafeERC20: ERC20 operation did n`D\x82\x01Ri\x1B\xDD\x08\x1C\xDDX\xD8\xD9YY`\xB2\x1B`d\x82\x01R`\x84\x01a\x13\x7FV[``a=4\x84\x84_\x85a=<V[\x94\x93PPPPV[``\x82G\x10\x15a=\x9DW`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`&`$\x82\x01R\x7FAddress: insufficient balance fo`D\x82\x01Re\x1C\x88\x18\xD8[\x1B`\xD2\x1B`d\x82\x01R`\x84\x01a\x13\x7FV[__\x86`\x01`\x01`\xA0\x1B\x03\x16\x85\x87`@Qa=\xB8\x91\x90aL8V[_`@Q\x80\x83\x03\x81\x85\x87Z\xF1\x92PPP=\x80_\x81\x14a=\xF2W`@Q\x91P`\x1F\x19`?=\x01\x16\x82\x01`@R=\x82R=_` \x84\x01>a=\xF7V[``\x91P[P\x91P\x91Pa\x12\xE6\x87\x83\x83\x87``\x83\x15a>qW\x82Q_\x03a>jW`\x01`\x01`\xA0\x1B\x03\x85\x16;a>jW`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`\x1D`$\x82\x01R\x7FAddress: call to non-contract\0\0\0`D\x82\x01R`d\x01a\x13\x7FV[P\x81a=4V[a=4\x83\x83\x81Q\x15a>\x86W\x81Q\x80\x83` \x01\xFD[\x80`@QbF\x1B\xCD`\xE5\x1B\x81R`\x04\x01a\x13\x7F\x91\x90aC\x1AV[a\x08\xB0\x80aM\x7F\x839\x01\x90V[\x805c\xFF\xFF\xFF\xFF\x81\x16\x81\x14a<PW__\xFD[_` \x82\x84\x03\x12\x15a>\xD0W__\xFD[a>\xD9\x82a>\xADV[\x93\x92PPPV[\x805`\x01`\x01`@\x1B\x03\x81\x16\x81\x14a<PW__\xFD[`\x01`\x01`\xA0\x1B\x03\x81\x16\x81\x14a'\xBBW__\xFD[\x805a<P\x81a>\xF6V[________a\x03\xE0\x89\x8B\x03\x12\x15a?-W__\xFD[a?6\x89a>\xADV[\x97Pa?D` \x8A\x01a>\xE0V[\x96Pa?R`@\x8A\x01a>\xE0V[\x95Pa?```\x8A\x01a>\xE0V[\x94P`\x80\x89\x015\x93P`\xA0\x89\x015\x92P`\xC0\x89\x015a?~\x81a>\xF6V[\x91Pa\x03\xE0\x89\x01\x8A\x10\x15a?\x90W__\xFD[`\xE0\x89\x01\x90P\x92\x95\x98P\x92\x95\x98\x90\x93\x96PV[_` \x82\x84\x03\x12\x15a?\xB3W__\xFD[P5\x91\x90PV[__`@\x83\x85\x03\x12\x15a?\xCBW__\xFD[a?\xD4\x83a>\xADV[\x91Pa?\xE2` \x84\x01a>\xE0V[\x90P\x92P\x92\x90PV[__`@\x83\x85\x03\x12\x15a?\xFCW__\xFD[\x825\x91P` \x83\x015a@\x0E\x81a>\xF6V[\x80\x91PP\x92P\x92\x90PV[cNH{q`\xE0\x1B_R`!`\x04R`$_\xFD[`\x03\x81\x10a@IWcNH{q`\xE0\x1B_R`!`\x04R`$_\xFD[\x90RV[`\x01`\x01`\xA0\x1B\x03\x88\x81\x16\x82R\x87\x16` \x82\x01R`\x01`\x01`@\x1B\x03\x86\x16`@\x82\x01R`\xE0\x81\x01a@\x81``\x83\x01\x87a@-V[\x93\x15\x15`\x80\x82\x01R`\xA0\x81\x01\x92\x90\x92R`\xC0\x90\x91\x01R\x94\x93PPPPV[__\x83`\x1F\x84\x01\x12a@\xAFW__\xFD[P\x815`\x01`\x01`@\x1B\x03\x81\x11\x15a@\xC5W__\xFD[` \x83\x01\x91P\x83` \x82\x85\x01\x01\x11\x15a@\xDCW__\xFD[\x92P\x92\x90PV[________`\xC0\x89\x8B\x03\x12\x15a@\xFAW__\xFD[aA\x03\x89a>\xADV[\x97PaA\x11` \x8A\x01a>\xADV[\x96P`@\x89\x015\x95P``\x89\x015\x94P`\x80\x89\x015`\x01`\x01`@\x1B\x03\x81\x11\x15aA9W__\xFD[aAE\x8B\x82\x8C\x01a@\x9FV[\x90\x95P\x93PP`\xA0\x89\x015`\x01`\x01`@\x1B\x03\x81\x11\x15aAcW__\xFD[aAo\x8B\x82\x8C\x01a@\x9FV[\x99\x9C\x98\x9BP\x96\x99P\x94\x97\x93\x96\x92\x95\x94PPPV[\x81Q`\x01`\x01`\xA0\x1B\x03\x16\x81Ra\x01\x80\x81\x01` \x83\x01QaA\xAF` \x84\x01\x82`\x01`\x01`@\x1B\x03\x16\x90RV[P`@\x83\x01QaA\xCA`@\x84\x01\x82`\x01`\x01`\xA0\x1B\x03\x16\x90RV[P``\x83\x01QaA\xE5``\x84\x01\x82`\x01`\x01`@\x1B\x03\x16\x90RV[P`\x80\x83\x01Q`\x80\x83\x01R`\xA0\x83\x01QaB\n`\xA0\x84\x01\x82`\x01`\x01`@\x1B\x03\x16\x90RV[P`\xC0\x83\x01QaB%`\xC0\x84\x01\x82`\x01`\x01`@\x1B\x03\x16\x90RV[P`\xE0\x83\x01QaB@`\xE0\x84\x01\x82`\x01`\x01`@\x1B\x03\x16\x90RV[Pa\x01\0\x83\x01QaB]a\x01\0\x84\x01\x82`\x01`\x01`@\x1B\x03\x16\x90RV[Pa\x01 \x83\x01QaBra\x01 \x84\x01\x82a@-V[Pa\x01@\x83\x01Qa\x01@\x83\x01Ra\x01`\x83\x01Qa\x01`\x83\x01R\x92\x91PPV[______`\xC0\x87\x89\x03\x12\x15aB\xA6W__\xFD[aB\xAF\x87a>\xADV[\x95PaB\xBD` \x88\x01a>\xE0V[\x94PaB\xCB`@\x88\x01a>\xE0V[\x95\x98\x94\x97P\x94\x95``\x81\x015\x95P`\x80\x81\x015\x94`\xA0\x90\x91\x015\x93P\x91PPV[_\x81Q\x80\x84R\x80` \x84\x01` \x86\x01^_` \x82\x86\x01\x01R` `\x1F\x19`\x1F\x83\x01\x16\x85\x01\x01\x91PP\x92\x91PPV[` \x81R_a>\xD9` \x83\x01\x84aB\xECV[_` \x82\x84\x03\x12\x15aC<W__\xFD[a>\xD9\x82a>\xE0V[__`@\x83\x85\x03\x12\x15aCVW__\xFD[\x825a?\xD4\x81a>\xF6V[__`@\x83\x85\x03\x12\x15aCrW__\xFD[aC{\x83a>\xE0V[\x94` \x93\x90\x93\x015\x93PPPV[\x805`\x03\x81\x10a<PW__\xFD[cNH{q`\xE0\x1B_R`A`\x04R`$_\xFD[_\x82`\x1F\x83\x01\x12aC\xBAW__\xFD[\x815` \x83\x01__`\x01`\x01`@\x1B\x03\x84\x11\x15aC\xD9WaC\xD9aC\x97V[P`@Q`\x1F\x19`\x1F\x85\x01\x81\x16`?\x01\x16\x81\x01\x81\x81\x10`\x01`\x01`@\x1B\x03\x82\x11\x17\x15aD\x07WaD\x07aC\x97V[`@R\x83\x81R\x90P\x80\x82\x84\x01\x87\x10\x15aD\x1EW__\xFD[\x83\x83` \x83\x017_` \x85\x83\x01\x01R\x80\x94PPPPP\x92\x91PPV[_______`\xE0\x88\x8A\x03\x12\x15aDPW__\xFD[\x875aD[\x81a>\xF6V[\x96P` \x88\x015aDk\x81a>\xF6V[\x95PaDy`@\x89\x01a>\xE0V[\x94PaD\x87``\x89\x01aC\x89V[\x93P`\x80\x88\x015\x92P`\xA0\x88\x015`\x01`\x01`@\x1B\x03\x81\x11\x15aD\xA8W__\xFD[aD\xB4\x8A\x82\x8B\x01aC\xABV[\x97\x9A\x96\x99P\x94\x97\x93\x96\x92\x95\x92\x94PPP`\xC0\x90\x91\x015\x90V[___``\x84\x86\x03\x12\x15aD\xDFW__\xFD[\x835aD\xEA\x81a>\xF6V[\x92PaD\xF8` \x85\x01a>\xADV[\x91P`@\x84\x015`\x01`\x01`@\x1B\x03\x81\x11\x15aE\x12W__\xFD[aE\x1E\x86\x82\x87\x01aC\xABV[\x91PP\x92P\x92P\x92V[________a\x01\0\x89\x8B\x03\x12\x15aE@W__\xFD[aEI\x89a>\xADV[\x97PaEW` \x8A\x01a>\xE0V[\x96PaEe`@\x8A\x01a?\nV[\x95PaEs``\x8A\x01a?\nV[\x94PaE\x81`\x80\x8A\x01a?\nV[\x93P`\xA0\x89\x015`\x01`\x01`@\x1B\x03\x81\x11\x15aE\x9BW__\xFD[aE\xA7\x8B\x82\x8C\x01aC\xABV[\x93PP`\xC0\x89\x015`\x01`\x01`@\x1B\x03\x81\x11\x15aE\xC2W__\xFD[aE\xCE\x8B\x82\x8C\x01aC\xABV[\x92PP`\xE0\x89\x015`\x01`\x01`@\x1B\x03\x81\x11\x15aE\xE9W__\xFD[aE\xF5\x8B\x82\x8C\x01aC\xABV[\x91PP\x92\x95\x98P\x92\x95\x98\x90\x93\x96PV[_` \x82\x84\x03\x12\x15aF\x15W__\xFD[\x815a>\xD9\x81a>\xF6V[______`\xA0\x87\x89\x03\x12\x15aF5W__\xFD[aF>\x87a>\xADV[\x95P` \x87\x015\x94P`@\x87\x015\x93P``\x87\x015\x92P`\x80\x87\x015`\x01`\x01`@\x1B\x03\x81\x11\x15aFmW__\xFD[aFy\x89\x82\x8A\x01a@\x9FV[\x97\x9A\x96\x99P\x94\x97P\x92\x95\x93\x94\x92PPPV[__`@\x83\x85\x03\x12\x15aF\x9CW__\xFD[\x825aF\xA7\x81a>\xF6V[\x91Pa?\xE2` \x84\x01a>\xADV[`\x01`\x01`\xA0\x1B\x03\x8D\x81\x16\x82R`\x01`\x01`@\x1B\x03\x8D\x81\x16` \x84\x01R\x90\x8C\x16`@\x83\x01R\x8A\x81\x16``\x83\x01R`\x80\x82\x01\x8A\x90R\x88\x81\x16`\xA0\x83\x01R\x87\x16`\xC0\x82\x01Ra\x01\x80\x81\x01`\x01`\x01`@\x1B\x03\x87\x16`\xE0\x83\x01R`\x01`\x01`@\x1B\x03\x86\x16a\x01\0\x83\x01RaG*a\x01 \x83\x01\x86a@-V[a\x01@\x82\x01\x93\x90\x93Ra\x01`\x01R\x9A\x99PPPPPPPPPPV[________a\x01\0\x89\x8B\x03\x12\x15aG^W__\xFD[\x885aGi\x81a>\xF6V[\x97P` \x89\x015aGy\x81a>\xF6V[\x96PaG\x87`@\x8A\x01a>\xE0V[\x95PaG\x95``\x8A\x01a>\xE0V[\x94P`\x80\x89\x015\x93PaG\xAA`\xA0\x8A\x01aC\x89V[\x97\x9A\x96\x99P\x94\x97\x93\x96\x92\x95\x92\x94PPP`\xC0\x82\x015\x91`\xE0\x015\x90V[\x81Q`\x01`\x01`\xA0\x1B\x03\x16\x81Ra\x01\x80\x81\x01` \x83\x01QaG\xF3` \x84\x01\x82`\x01`\x01`@\x1B\x03\x16\x90RV[P`@\x83\x01QaH\x0E`@\x84\x01\x82`\x01`\x01`\xA0\x1B\x03\x16\x90RV[P``\x83\x01QaH)``\x84\x01\x82`\x01`\x01`@\x1B\x03\x16\x90RV[P`\x80\x83\x01Q`\x80\x83\x01R`\xA0\x83\x01QaHN`\xA0\x84\x01\x82`\x01`\x01`@\x1B\x03\x16\x90RV[P`\xC0\x83\x01QaHi`\xC0\x84\x01\x82`\x01`\x01`@\x1B\x03\x16\x90RV[P`\xE0\x83\x01QaH\x84`\xE0\x84\x01\x82`\x01`\x01`@\x1B\x03\x16\x90RV[Pa\x01\0\x83\x01QaH\xA1a\x01\0\x84\x01\x82`\x01`\x01`@\x1B\x03\x16\x90RV[Pa\x01 \x83\x01QaH\xBEa\x01 \x84\x01\x82`\x01`\x01`@\x1B\x03\x16\x90RV[Pa\x01@\x83\x01QaH\xDBa\x01@\x84\x01\x82`\x01`\x01`@\x1B\x03\x16\x90RV[Pa\x01`\x83\x01QaH\xF0a\x01`\x84\x01\x82a@-V[P\x92\x91PPV[cNH{q`\xE0\x1B_R`\x11`\x04R`$_\xFD[`\x01`\x01`@\x1B\x03\x81\x81\x16\x83\x82\x16\x01\x90\x81\x11\x15a\tcWa\tcaH\xF7V[\x80\x82\x02\x81\x15\x82\x82\x04\x84\x14\x17a\tcWa\tcaH\xF7V[_` \x82\x84\x03\x12\x15aIQW__\xFD[PQ\x91\x90PV[\x81\x83R\x81\x81` \x85\x017P_\x82\x82\x01` \x90\x81\x01\x91\x90\x91R`\x1F\x90\x91\x01`\x1F\x19\x16\x90\x91\x01\x01\x90V[`@\x81R_aI\x92`@\x83\x01\x86aB\xECV[\x82\x81\x03` \x84\x01RaI\xA5\x81\x85\x87aIXV[\x96\x95PPPPPPV[\x84\x81R``` \x82\x01R_aI\xC7``\x83\x01\x86aB\xECV[\x82\x81\x03`@\x84\x01Ra\x12\xE6\x81\x85\x87aIXV[` \x81R_a=4` \x83\x01\x84\x86aIXV[_` \x82\x84\x03\x12\x15aI\xFDW__\xFD[\x81Qa>\xD9\x81a>\xF6V[`\x01`\x01`@\x1B\x03\x82\x81\x16\x82\x82\x16\x03\x90\x81\x11\x15a\tcWa\tcaH\xF7V[cNH{q`\xE0\x1B_R`\x12`\x04R`$_\xFD[_\x82aJIWaJIaJ'V[P\x04\x90V[\x80\x82\x01\x80\x82\x11\x15a\tcWa\tcaH\xF7V[cNH{q`\xE0\x1B_R`2`\x04R`$_\xFD[_\x82aJ\x83WaJ\x83aJ'V[P\x06\x90V[\x81\x81\x03\x81\x81\x11\x15a\tcWa\tcaH\xF7V[_\x81aJ\xA9WaJ\xA9aH\xF7V[P_\x19\x01\x90V[_c\xFF\xFF\xFF\xFF\x82\x16c\xFF\xFF\xFF\xFF\x81\x03aJ\xCBWaJ\xCBaH\xF7V[`\x01\x01\x92\x91PPV[`\x01`\x01`\xA0\x1B\x03\x88\x16\x81R`\x01`\x01`\xA0\x1B\x03\x87\x16` \x82\x01R`\x01`\x01`@\x1B\x03\x86\x16`@\x82\x01RaK\x0B``\x82\x01\x86a@-V[\x83`\x80\x82\x01R`\xE0`\xA0\x82\x01R_aK&`\xE0\x83\x01\x85aB\xECV[\x90P\x82`\xC0\x83\x01R\x98\x97PPPPPPPPV[`\x01`\x01`\xA0\x1B\x03\x84\x16\x81R`\x01`\x01`\xA0\x1B\x03\x83\x16` \x82\x01R```@\x82\x01R_aKj``\x83\x01\x84aB\xECV[\x95\x94PPPPPV[`\x01`\x01`\xA0\x1B\x03\x87\x16\x81R`\x01`\x01`\xA0\x1B\x03\x86\x16` \x82\x01Rc\xFF\xFF\xFF\xFF\x85\x16`@\x82\x01R`\x01`\x01`\xA0\x1B\x03\x84\x16``\x82\x01R`\xC0`\x80\x82\x01R_aK\xBE`\xC0\x83\x01\x85aB\xECV[\x82\x81\x03`\xA0\x84\x01RaK\xD0\x81\x85aB\xECV[\x99\x98PPPPPPPPPV[`\x01`\x01`@\x1B\x03\x88\x81\x16\x82R`\x01`\x01`\xA0\x1B\x03\x88\x16` \x83\x01R\x86\x16`@\x82\x01R`\xE0\x81\x01aL\x11``\x83\x01\x87a@-V[`\x01`\x01`@\x1B\x03\x85\x16`\x80\x83\x01R\x83`\xA0\x83\x01R\x82`\xC0\x83\x01R\x98\x97PPPPPPPPV[_\x82Q\x80` \x85\x01\x84^_\x92\x01\x91\x82RP\x91\x90PV[a\x03 \x81\x01a\x03\0\x84\x837a\x03\0\x82\x01\x83_[`\x01\x81\x10\x15aL\x80W\x81Q\x83R` \x92\x83\x01\x92\x90\x91\x01\x90`\x01\x01aLaV[PPP\x93\x92PPPV[_` \x82\x84\x03\x12\x15aL\x9AW__\xFD[\x81Q\x80\x15\x15\x81\x14a>\xD9W__\xFD[k\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x19\x8B``\x1B\x16\x81R\x89`\x14\x82\x01R\x88`4\x82\x01R`\x01`\x01`@\x1B\x03`\xC0\x1B\x88`\xC0\x1B\x16`T\x82\x01R`\x01`\x01`@\x1B\x03`\xC0\x1B\x87`\xC0\x1B\x16`\\\x82\x01R`\x01`\x01`@\x1B\x03`\xC0\x1B\x86`\xC0\x1B\x16`d\x82\x01R\x84`l\x82\x01R\x83`\x8C\x82\x01R\x82`\xAC\x82\x01RaML`\xCC\x82\x01\x83`\xC0\x1B\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x16\x90RV[`\xD4\x01\x9A\x99PPPPPPPPPPV[`\x01`\x01`\xA0\x1B\x03\x83\x16\x81R`@` \x82\x01R_a=4`@\x83\x01\x84aB\xECV\xFE`\xA0`@R`@Qa\x08\xB08\x03\x80a\x08\xB0\x839\x81\x01`@\x81\x90Ra\0\"\x91a\x03'V[\x82\x81a\0.\x82\x82a\0VV[PP`\x01`\x01`\xA0\x1B\x03\x82\x16`\x80Ra\0Na\0I`\x80Q\x90V[a\0\xB4V[PPPa\x04\x0EV[a\0_\x82a\x01!V[`@Q`\x01`\x01`\xA0\x1B\x03\x83\x16\x90\x7F\xBC|\xD7Z \xEE'\xFD\x9A\xDE\xBA\xB3 A\xF7U!M\xBCk\xFF\xA9\x0C\xC0\"[9\xDA.\\-;\x90_\x90\xA2\x80Q\x15a\0\xA8Wa\0\xA3\x82\x82a\x01\x9FV[PPPV[a\0\xB0a\x02\x12V[PPV[\x7F~dMyB/\x17\xC0\x1EH\x94\xB5\xF4\xF5\x88\xD31\xEB\xFA(e=B\xAE\x83-\xC5\x9E8\xC9y\x8Fa\0\xF3_Q` a\x08\x90_9_Q\x90_RT`\x01`\x01`\xA0\x1B\x03\x16\x90V[`@\x80Q`\x01`\x01`\xA0\x1B\x03\x92\x83\x16\x81R\x91\x84\x16` \x83\x01R\x01`@Q\x80\x91\x03\x90\xA1a\x01\x1E\x81a\x023V[PV[\x80`\x01`\x01`\xA0\x1B\x03\x16;_\x03a\x01[W`@QcL\x9C\x8C\xE3`\xE0\x1B\x81R`\x01`\x01`\xA0\x1B\x03\x82\x16`\x04\x82\x01R`$\x01[`@Q\x80\x91\x03\x90\xFD[\x80\x7F6\x08\x94\xA1;\xA1\xA3!\x06g\xC8(I-\xB9\x8D\xCA> v\xCC75\xA9 \xA3\xCAP]8+\xBC[\x80T`\x01`\x01`\xA0\x1B\x03\x19\x16`\x01`\x01`\xA0\x1B\x03\x92\x90\x92\x16\x91\x90\x91\x17\x90UPV[``__\x84`\x01`\x01`\xA0\x1B\x03\x16\x84`@Qa\x01\xBB\x91\x90a\x03\xF8V[_`@Q\x80\x83\x03\x81\x85Z\xF4\x91PP=\x80_\x81\x14a\x01\xF3W`@Q\x91P`\x1F\x19`?=\x01\x16\x82\x01`@R=\x82R=_` \x84\x01>a\x01\xF8V[``\x91P[P\x90\x92P\x90Pa\x02\t\x85\x83\x83a\x02pV[\x95\x94PPPPPV[4\x15a\x021W`@Qc\xB3\x98\x97\x9F`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[V[`\x01`\x01`\xA0\x1B\x03\x81\x16a\x02\\W`@Qc1s\xBD\xD1`\xE1\x1B\x81R_`\x04\x82\x01R`$\x01a\x01RV[\x80_Q` a\x08\x90_9_Q\x90_Ra\x01~V[``\x82a\x02\x85Wa\x02\x80\x82a\x02\xCFV[a\x02\xC8V[\x81Q\x15\x80\x15a\x02\x9CWP`\x01`\x01`\xA0\x1B\x03\x84\x16;\x15[\x15a\x02\xC5W`@Qc\x99\x96\xB3\x15`\xE0\x1B\x81R`\x01`\x01`\xA0\x1B\x03\x85\x16`\x04\x82\x01R`$\x01a\x01RV[P\x80[\x93\x92PPPV[\x80Q\x15a\x02\xDFW\x80Q\x80\x82` \x01\xFD[`@Qc\n\x12\xF5!`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x80Q`\x01`\x01`\xA0\x1B\x03\x81\x16\x81\x14a\x03\x0EW__\xFD[\x91\x90PV[cNH{q`\xE0\x1B_R`A`\x04R`$_\xFD[___``\x84\x86\x03\x12\x15a\x039W__\xFD[a\x03B\x84a\x02\xF8V[\x92Pa\x03P` \x85\x01a\x02\xF8V[`@\x85\x01Q\x90\x92P`\x01`\x01`@\x1B\x03\x81\x11\x15a\x03kW__\xFD[\x84\x01`\x1F\x81\x01\x86\x13a\x03{W__\xFD[\x80Q`\x01`\x01`@\x1B\x03\x81\x11\x15a\x03\x94Wa\x03\x94a\x03\x13V[`@Q`\x1F\x82\x01`\x1F\x19\x90\x81\x16`?\x01\x16\x81\x01`\x01`\x01`@\x1B\x03\x81\x11\x82\x82\x10\x17\x15a\x03\xC2Wa\x03\xC2a\x03\x13V[`@R\x81\x81R\x82\x82\x01` \x01\x88\x10\x15a\x03\xD9W__\xFD[\x81` \x84\x01` \x83\x01^_` \x83\x83\x01\x01R\x80\x93PPPP\x92P\x92P\x92V[_\x82Q\x80` \x85\x01\x84^_\x92\x01\x91\x82RP\x91\x90PV[`\x80Qa\x04ka\x04%_9_`\x10\x01Ra\x04k_\xF3\xFE`\x80`@Ra\0\x0Ca\0\x0EV[\0[\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`\x01`\x01`\xA0\x1B\x03\x163\x03a\0\x81W_5\x7F\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x16c'\x8FyC`\xE1\x1B\x14a\0yWa\0wa\0\x85V[V[a\0wa\0\x95V[a\0w[a\0wa\0\x90a\0\xC3V[a\0\xFAV[_\x80a\0\xA46`\x04\x81\x84a\x03\x13V[\x81\x01\x90a\0\xB1\x91\x90a\x03NV[\x91P\x91Pa\0\xBF\x82\x82a\x01\x18V[PPV[_a\0\xF5\x7F6\x08\x94\xA1;\xA1\xA3!\x06g\xC8(I-\xB9\x8D\xCA> v\xCC75\xA9 \xA3\xCAP]8+\xBCT`\x01`\x01`\xA0\x1B\x03\x16\x90V[\x90P\x90V[6__7__6_\x84Z\xF4=__>\x80\x80\x15a\x01\x14W=_\xF3[=_\xFD[a\x01!\x82a\x01rV[`@Q`\x01`\x01`\xA0\x1B\x03\x83\x16\x90\x7F\xBC|\xD7Z \xEE'\xFD\x9A\xDE\xBA\xB3 A\xF7U!M\xBCk\xFF\xA9\x0C\xC0\"[9\xDA.\\-;\x90_\x90\xA2\x80Q\x15a\x01jWa\x01e\x82\x82a\x01\xFAV[PPPV[a\0\xBFa\x02lV[\x80`\x01`\x01`\xA0\x1B\x03\x16;_\x03a\x01\xACW`@QcL\x9C\x8C\xE3`\xE0\x1B\x81R`\x01`\x01`\xA0\x1B\x03\x82\x16`\x04\x82\x01R`$\x01[`@Q\x80\x91\x03\x90\xFD[\x7F6\x08\x94\xA1;\xA1\xA3!\x06g\xC8(I-\xB9\x8D\xCA> v\xCC75\xA9 \xA3\xCAP]8+\xBC\x80Ts\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x19\x16`\x01`\x01`\xA0\x1B\x03\x92\x90\x92\x16\x91\x90\x91\x17\x90UV[``__\x84`\x01`\x01`\xA0\x1B\x03\x16\x84`@Qa\x02\x16\x91\x90a\x04\x1FV[_`@Q\x80\x83\x03\x81\x85Z\xF4\x91PP=\x80_\x81\x14a\x02NW`@Q\x91P`\x1F\x19`?=\x01\x16\x82\x01`@R=\x82R=_` \x84\x01>a\x02SV[``\x91P[P\x91P\x91Pa\x02c\x85\x83\x83a\x02\x8BV[\x95\x94PPPPPV[4\x15a\0wW`@Qc\xB3\x98\x97\x9F`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[``\x82a\x02\xA0Wa\x02\x9B\x82a\x02\xEAV[a\x02\xE3V[\x81Q\x15\x80\x15a\x02\xB7WP`\x01`\x01`\xA0\x1B\x03\x84\x16;\x15[\x15a\x02\xE0W`@Qc\x99\x96\xB3\x15`\xE0\x1B\x81R`\x01`\x01`\xA0\x1B\x03\x85\x16`\x04\x82\x01R`$\x01a\x01\xA3V[P\x80[\x93\x92PPPV[\x80Q\x15a\x02\xFAW\x80Q\x80\x82` \x01\xFD[`@Qc\n\x12\xF5!`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[__\x85\x85\x11\x15a\x03!W__\xFD[\x83\x86\x11\x15a\x03-W__\xFD[PP\x82\x01\x93\x91\x90\x92\x03\x91PV[cNH{q`\xE0\x1B_R`A`\x04R`$_\xFD[__`@\x83\x85\x03\x12\x15a\x03_W__\xFD[\x825`\x01`\x01`\xA0\x1B\x03\x81\x16\x81\x14a\x03uW__\xFD[\x91P` \x83\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x03\x90W__\xFD[\x83\x01`\x1F\x81\x01\x85\x13a\x03\xA0W__\xFD[\x805g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x03\xBAWa\x03\xBAa\x03:V[`@Q`\x1F\x82\x01`\x1F\x19\x90\x81\x16`?\x01\x16\x81\x01g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x82\x82\x10\x17\x15a\x03\xE9Wa\x03\xE9a\x03:V[`@R\x81\x81R\x82\x82\x01` \x01\x87\x10\x15a\x04\0W__\xFD[\x81` \x84\x01` \x83\x017_` \x83\x83\x01\x01R\x80\x93PPPP\x92P\x92\x90PV[_\x82Q\x80` \x85\x01\x84^_\x92\x01\x91\x82RP\x91\x90PV\xFE\xA2dipfsX\"\x12 \xF0\xC4\x0B\x8D\x0B\xC5ZE\x01:q\xEE?\xD7&L\xE8s\xFC\xC5a\xD0JA\xBA\x1F$\xAD\xBC\xEF\x95\x8CdsolcC\0\x08\x1C\x003\xB51'hJV\x8B1s\xAE\x13\xB9\xF8\xA6\x01n$>c\xB6\xE8\xEE\x11x\xD6\xA7\x17\x85\x0B]a\x03\xA2dipfsX\"\x12 :<\x88\xF2\x08\xED\xABu\xCBuw\xE6j\xB9\x8D\xC6\x12\xD1\xC8\x03\x8C\xB4\xBD\xFD\xA5\xCC\\\xF9G\xA1\xD8\x91dsolcC\0\x08\x1C\x003";
    /// The bytecode of the contract.
    pub static POLYGONROLLUPMANAGER_BYTECODE: ::ethers::core::types::Bytes = ::ethers::core::types::Bytes::from_static(
        __BYTECODE,
    );
    #[rustfmt::skip]
    const __DEPLOYED_BYTECODE: &[u8] = b"`\x80`@R4\x80\x15a\0\x0FW__\xFD[P`\x046\x10a\x02\xF5W_5`\xE0\x1C\x80c\x99\xF5cN\x11a\x01\x9DW\x80c\xD5\x07?o\x11a\0\xE8W\x80c\xDF\xDB\x8C^\x11a\0\x93W\x80c\xE8\x0EP0\x11a\0nW\x80c\xE8\x0EP0\x14a\x08\xF7W\x80c\xF4\xE9&u\x14a\t\nW\x80c\xF9\xC4\xC2\xAE\x14a\t\x1AW__\xFD[\x80c\xDF\xDB\x8C^\x14a\x08\x0FW\x80c\xE4ga\xC4\x14a\x08\"W\x80c\xE4\xF3\xD8\xF9\x14a\x08IW__\xFD[\x80c\xDB\xC1iv\x11a\0\xC3W\x80c\xDB\xC1iv\x14a\x07\xDAW\x80c\xDD\x04d\xB9\x14a\x07\xE2W\x80c\xDD\xE0\xFFw\x14a\x07\xF5W__\xFD[\x80c\xD5\x07?o\x14a\x07\x8CW\x80c\xD5Gt\x1F\x14a\x07\x9FW\x80c\xD8\x90X\x12\x14a\x07\xB2W__\xFD[\x80c\xAB\xCBQ\x98\x11a\x01HW\x80c\xC5\xB4\xFD\xB6\x11a\x01#W\x80c\xC5\xB4\xFD\xB6\x14a\x07-W\x80c\xCE\xEE(\x1D\x14a\x07@W\x80c\xD0!\x03\xCA\x14a\x07eW__\xFD[\x80c\xAB\xCBQ\x98\x14a\x06\xEDW\x80c\xC1\xAC\xBC4\x14a\x07\0W\x80c\xC4\xC9(\xC2\x14a\x07\x1AW__\xFD[\x80c\xA2\x96}\x99\x11a\x01xW\x80c\xA2\x96}\x99\x14a\x06\x7FW\x80c\xA3\xC5s\xEB\x14a\x06\x87W\x80c\xAB\x04u\xCF\x14a\x06\xC6W__\xFD[\x80c\x99\xF5cN\x14a\x06]W\x80c\x9A\x90\x8Es\x14a\x06eW\x80c\xA2\x17\xFD\xDF\x14a\x06xW__\xFD[\x80cG\x7F\xA2p\x11a\x02]W\x80ct\xD9\xC2D\x11a\x02\x08W\x80c\x81)\xFC\x1C\x11a\x01\xE3W\x80c\x81)\xFC\x1C\x14a\x06\nW\x80c\x8F\xD8\x8C\xC2\x14a\x06\x12W\x80c\x91\xD1HT\x14a\x06%W__\xFD[\x80ct\xD9\xC2D\x14a\x05\xA5W\x80cyu\xFC\xFE\x14a\x05\xC5W\x80c\x7F\xB6\xE7j\x14a\x05\xE5W__\xFD[\x80ce\xC0PM\x11a\x028W\x80ce\xC0PM\x14a\x05\x04W\x80clvhw\x14a\x05\x7FW\x80cr\"\x02\x0F\x14a\x05\x92W__\xFD[\x80cG\x7F\xA2p\x14a\x04\xB4W\x80cU\xA7\x1E\xE0\x14a\x04\xBCW\x80c`F\x91i\x14a\x04\xFCW__\xFD[\x80c r\xF6\xC5\x11a\x02\xBDW\x80c//\xF1]\x11a\x02\x98W\x80c//\xF1]\x14a\x04{W\x80c0\xC2}\xDE\x14a\x04\x8EW\x80c6V\x8A\xBE\x14a\x04\xA1W__\xFD[\x80c r\xF6\xC5\x14a\x03\x93W\x80c$\x8A\x9C\xA3\x14a\x03\x9BW\x80c%(\x01i\x14a\x03\xCBW__\xFD[\x80c\x06n\xC0\x12\x14a\x02\xF9W\x80c\x11\xF6\xB2\x87\x14a\x03)W\x80c\x14\x89\xED\x10\x14a\x03<W\x80c\x15\x06L\x96\x14a\x03QW\x80c\x17\x96\xA1\xAE\x14a\x03nW[__\xFD[`\x84Ta\x03\x0C\x90`\x01`\x01`@\x1B\x03\x16\x81V[`@Q`\x01`\x01`@\x1B\x03\x90\x91\x16\x81R` \x01[`@Q\x80\x91\x03\x90\xF3[a\x03\x0Ca\x0376`\x04a>\xC0V[a\t:V[a\x03Oa\x03J6`\x04a?\x15V[a\tiV[\0[`oTa\x03^\x90`\xFF\x16\x81V[`@Q\x90\x15\x15\x81R` \x01a\x03 V[`~Ta\x03~\x90c\xFF\xFF\xFF\xFF\x16\x81V[`@Qc\xFF\xFF\xFF\xFF\x90\x91\x16\x81R` \x01a\x03 V[a\x03Oa\x0BJV[a\x03\xBDa\x03\xA96`\x04a?\xA3V[_\x90\x81R`4` R`@\x90 `\x01\x01T\x90V[`@Q\x90\x81R` \x01a\x03 V[a\x04Ha\x03\xD96`\x04a?\xBAV[`@\x80Q``\x80\x82\x01\x83R_\x80\x83R` \x80\x84\x01\x82\x90R\x92\x84\x01\x81\x90Rc\xFF\xFF\xFF\xFF\x95\x90\x95\x16\x85R`\x81\x82R\x82\x85 `\x01`\x01`@\x1B\x03\x94\x85\x16\x86R`\x03\x01\x82R\x93\x82\x90 \x82Q\x94\x85\x01\x83R\x80T\x85R`\x01\x01T\x80\x84\x16\x91\x85\x01\x91\x90\x91R`\x01`@\x1B\x90\x04\x90\x91\x16\x90\x82\x01R\x90V[`@\x80Q\x82Q\x81R` \x80\x84\x01Q`\x01`\x01`@\x1B\x03\x90\x81\x16\x91\x83\x01\x91\x90\x91R\x92\x82\x01Q\x90\x92\x16\x90\x82\x01R``\x01a\x03 V[a\x03Oa\x04\x896`\x04a?\xEBV[a\x0C\x1CV[`\x87Ta\x03\x0C\x90`\x01`\x01`@\x1B\x03\x16\x81V[a\x03Oa\x04\xAF6`\x04a?\xEBV[a\x0CEV[`\x86Ta\x03\xBDV[a\x03\xBDa\x04\xCA6`\x04a?\xBAV[c\xFF\xFF\xFF\xFF\x82\x16_\x90\x81R`\x81` \x90\x81R`@\x80\x83 `\x01`\x01`@\x1B\x03\x85\x16\x84R`\x02\x01\x90\x91R\x90 T\x92\x91PPV[a\x03\xBDa\x0C|V[a\x05la\x05\x126`\x04a>\xC0V[`\x7F` R_\x90\x81R`@\x90 \x80T`\x01\x82\x01T`\x02\x83\x01T`\x03\x90\x93\x01T`\x01`\x01`\xA0\x1B\x03\x92\x83\x16\x93\x92\x82\x16\x92`\x01`\x01`@\x1B\x03`\x01`\xA0\x1B\x84\x04\x16\x92`\xFF`\x01`\xE0\x1B\x82\x04\x81\x16\x93`\x01`\xE8\x1B\x90\x92\x04\x16\x91\x90\x87V[`@Qa\x03 \x97\x96\x95\x94\x93\x92\x91\x90a@MV[a\x03Oa\x05\x8D6`\x04a@\xE3V[a\x0C\x91V[a\x03Oa\x05\xA06`\x04a>\xC0V[a\x10\x81V[a\x05\xB8a\x05\xB36`\x04a>\xC0V[a\x11rV[`@Qa\x03 \x91\x90aA\x83V[a\x05\xD8a\x05\xD36`\x04aB\x91V[a\x12\xC1V[`@Qa\x03 \x91\x90aC\x1AV[a\x03~a\x05\xF36`\x04aC,V[`\x83` R_\x90\x81R`@\x90 Tc\xFF\xFF\xFF\xFF\x16\x81V[a\x03Oa\x12\xF1V[a\x03Oa\x06 6`\x04aCEV[a\x142V[a\x03^a\x0636`\x04a?\xEBV[_\x91\x82R`4` \x90\x81R`@\x80\x84 `\x01`\x01`\xA0\x1B\x03\x93\x90\x93\x16\x84R\x91\x90R\x90 T`\xFF\x16\x90V[a\x03\xBDa\x17\xB7V[a\x03\x0Ca\x06s6`\x04aCaV[a\x18\x90V[a\x03\xBD_\x81V[a\x03\xBDa\x1A\x86V[a\x06\xAE\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81V[`@Q`\x01`\x01`\xA0\x1B\x03\x90\x91\x16\x81R` \x01a\x03 V[a\x06\xAE\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81V[a\x03Oa\x06\xFB6`\x04aD:V[a\x1D\xECV[`\x84Ta\x03\x0C\x90`\x01`\x80\x1B\x90\x04`\x01`\x01`@\x1B\x03\x16\x81V[a\x03Oa\x07(6`\x04aD\xCDV[a \xE4V[a\x03Oa\x07;6`\x04aE(V[a!\x1FV[a\x03~a\x07N6`\x04aF\x05V[`\x82` R_\x90\x81R`@\x90 Tc\xFF\xFF\xFF\xFF\x16\x81V[a\x06\xAE\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81V[a\x03Oa\x07\x9A6`\x04a?\xA3V[a&6V[a\x03Oa\x07\xAD6`\x04a?\xEBV[a&\xD4V[a\x05\xD8`@Q\x80`@\x01`@R\x80`\t\x81R` \x01h\x06\x16\xC2\xD7c\x02\xE32\xE3`\xBC\x1B\x81RP\x81V[a\x03Oa&\xF8V[a\x05\xD8a\x07\xF06`\x04aF V[a'\xBEV[`\x84Ta\x03\x0C\x90`\x01`@\x1B\x90\x04`\x01`\x01`@\x1B\x03\x16\x81V[a\x03Oa\x08\x1D6`\x04aF\x8BV[a'\xE5V[a\x06\xAE\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81V[a\x08\xDFa\x08W6`\x04a>\xC0V[c\xFF\xFF\xFF\xFF\x16_\x90\x81R`\x81` R`@\x90 \x80T`\x01\x82\x01T`\x05\x83\x01T`\x06\x84\x01T`\x07\x85\x01T`\x08\x86\x01T`\t\x90\x96\x01T`\x01`\x01`\xA0\x1B\x03\x80\x87\x16\x98`\x01`\xA0\x1B\x97\x88\x90\x04`\x01`\x01`@\x1B\x03\x90\x81\x16\x99\x92\x88\x16\x98\x90\x97\x04\x87\x16\x96\x80\x86\x16\x95`\x01`@\x1B\x90\x81\x90\x04\x82\x16\x95\x82\x81\x16\x95\x91\x81\x04\x90\x92\x16\x93`\x01`\x80\x1B\x90\x92\x04`\xFF\x16\x92V[`@Qa\x03 \x9C\x9B\x9A\x99\x98\x97\x96\x95\x94\x93\x92\x91\x90aF\xB5V[a\x03Oa\t\x056`\x04aGFV[a*\rV[`\x80Ta\x03~\x90c\xFF\xFF\xFF\xFF\x16\x81V[a\t-a\t(6`\x04a>\xC0V[a-CV[`@Qa\x03 \x91\x90aG\xC7V[c\xFF\xFF\xFF\xFF\x81\x16_\x90\x81R`\x81` R`@\x81 `\x06\x01T`\x01`@\x1B\x90\x04`\x01`\x01`@\x1B\x03\x16[\x92\x91PPV[\x7F\x08N\x94\xF3u\xE9\xD6G\xF8\x7F[,\xEF\xFB\xA1\xE0b\xC7\x0F`\t\xFD\xBC\xF8\x02\x91\xE8\x03\xB5\xC9\xED\xD4a\t\x93\x81a.\x99V[`\x01`\x01`@\x1B\x03\x88\x16\x15a\t\xBBW`@Qc0m\xFCW`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[c\xFF\xFF\xFF\xFF\x89\x16_\x90\x81R`\x81` R`@\x81 \x90`\x07\x82\x01T`\x01`\x80\x1B\x90\x04`\xFF\x16`\x02\x81\x11\x15a\t\xF0Wa\t\xF0a@\x19V[\x14a\n\x0EW`@Qc\x90\xDB\r\x07`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\n\x1D\x81\x89\x89\x89\x89\x89\x89a.\xA3V[`\x06\x81\x01\x80To\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\x19\x16`\x01`@\x1B`\x01`\x01`@\x1B\x03\x8A\x16\x90\x81\x02\x91\x90\x91\x17\x90\x91U_\x90\x81R`\x02\x82\x01` R`@\x90 \x85\x90U`\x05\x81\x01\x86\x90U\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`\x01`\x01`\xA0\x1B\x03\x16c3\xD6$}a\n\x9Fa\x1A\x86V[`@Q\x82c\xFF\xFF\xFF\xFF\x16`\xE0\x1B\x81R`\x04\x01a\n\xBD\x91\x81R` \x01\x90V[_`@Q\x80\x83\x03\x81_\x87\x80;\x15\x80\x15a\n\xD4W__\xFD[PZ\xF1\x15\x80\x15a\n\xE6W=__>=_\xFD[PP`@\x80Q`\x01`\x01`@\x1B\x03\x8B\x16\x81R` \x81\x01\x89\x90R\x90\x81\x01\x89\x90R3\x92Pc\xFF\xFF\xFF\xFF\x8D\x16\x91P\x7F\xD1\xEC:\x12\x16\xF0\x8Bn\xFFr\xE1i\xCE\xB5H\xB7\x82\xDB\x18\xA6aHRa\x8D\x86\xBB\x19\xF3\xF9\xB0\xD3\x90``\x01`@Q\x80\x91\x03\x90\xA3PPPPPPPPPPV[3_\x90\x81R\x7F\x88u\xB9J\xF5ez)\x03\xDE\xF9\x90mg\xA3\xF4-\x8A\x83m$\xB5`,\0\xF0\x0F\xC8U3\x9F\xCD` R`@\x90 T`\xFF\x16a\x0C\x12W`\x84T`\x01`\x80\x1B\x90\x04`\x01`\x01`@\x1B\x03\x16\x15\x80a\x0B\xC7WP`\x84TB\x90a\x0B\xBC\x90b\t:\x80\x90`\x01`\x80\x1B\x90\x04`\x01`\x01`@\x1B\x03\x16aI\x0BV[`\x01`\x01`@\x1B\x03\x16\x11[\x80a\x0B\xF4WP`\x87TB\x90a\x0B\xE9\x90b\t:\x80\x90`\x01`\x01`@\x1B\x03\x16aI\x0BV[`\x01`\x01`@\x1B\x03\x16\x11[\x15a\x0C\x12W`@Qci+\xAA\xAD`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\x0C\x1Aa1\xF1V[V[_\x82\x81R`4` R`@\x90 `\x01\x01Ta\x0C6\x81a.\x99V[a\x0C@\x83\x83a2gV[PPPV[`\x01`\x01`\xA0\x1B\x03\x81\x163\x14a\x0CnW`@Qc\x0BJ\xD1\xCD`\xE3\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\x0Cx\x82\x82a2\xEAV[PPV[_`\x86T`da\x0C\x8C\x91\x90aI*V[\x90P\x90V[\x7F\x08N\x94\xF3u\xE9\xD6G\xF8\x7F[,\xEF\xFB\xA1\xE0b\xC7\x0F`\t\xFD\xBC\xF8\x02\x91\xE8\x03\xB5\xC9\xED\xD4a\x0C\xBB\x81a.\x99V[a\x0C\xC3a3kV[c\xFF\xFF\xFF\xFF\x89\x16_\x90\x81R`\x81` R`@\x81 \x90`\x07\x82\x01T`\x01`\x80\x1B\x90\x04`\xFF\x16`\x02\x81\x11\x15a\x0C\xF8Wa\x0C\xF8a@\x19V[\x03a\r\x16W`@Qc[f\x02\xB7`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`@Qc\xEFN\xEB5`\xE0\x1B\x81Rc\xFF\xFF\xFF\xFF\x8A\x16`\x04\x82\x01R_\x90\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`\x01`\x01`\xA0\x1B\x03\x16\x90c\xEFN\xEB5\x90`$\x01` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a\r\x80W=__>=_\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\r\xA4\x91\x90aIAV[\x90P\x80a\r\xC4W`@Qc\xA6\x07!\xE1`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[_a\r\xD4\x8C\x84\x84\x8D\x8D\x8B\x8Ba3\xD8V[\x90P`\x02`\x07\x84\x01T`\x01`\x80\x1B\x90\x04`\xFF\x16`\x02\x81\x11\x15a\r\xF8Wa\r\xF8a@\x19V[\x03a\x0E~W`@Qc\xA4\x8F\xD3w`\xE0\x1B\x81R`\x01`\x01`\xA0\x1B\x03\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x16\x90c\xA4\x8F\xD3w\x90a\x0EM\x90\x84\x90\x8C\x90\x8C\x90`\x04\x01aI\x80V[_`@Q\x80\x83\x03\x81\x86\x80;\x15\x80\x15a\x0EcW__\xFD[PZ\xFA\x15\x80\x15a\x0EuW=__>=_\xFD[PPPPa\x0E\xE6V[`\x01\x83\x01T`\t\x84\x01T`@Qc\x02\nI\xE3`\xE5\x1B\x81R`\x01`\x01`\xA0\x1B\x03\x90\x92\x16\x91cAI<`\x91a\x0E\xB9\x91\x85\x90\x8D\x90\x8D\x90`\x04\x01aI\xAFV[_`@Q\x80\x83\x03\x81\x86\x80;\x15\x80\x15a\x0E\xCFW__\xFD[PZ\xFA\x15\x80\x15a\x0E\xE1W=__>=_\xFD[PPPP[`\x84\x80Tg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF`\x80\x1B\x19\x16`\x01`\x80\x1BB`\x01`\x01`@\x1B\x03\x16\x02\x17\x90U`\x05\x83\x01\x8A\x90U`\x08\x83\x01\x89\x90U\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`\x01`\x01`\xA0\x1B\x03\x16c3\xD6$}a\x0FPa\x1A\x86V[`@Q\x82c\xFF\xFF\xFF\xFF\x16`\xE0\x1B\x81R`\x04\x01a\x0Fn\x91\x81R` \x01\x90V[_`@Q\x80\x83\x03\x81_\x87\x80;\x15\x80\x15a\x0F\x85W__\xFD[PZ\xF1\x15\x80\x15a\x0F\x97W=__>=_\xFD[PP`@\x80Q_\x80\x82R` \x82\x01R\x90\x81\x01\x8D\x90R3\x92Pc\xFF\xFF\xFF\xFF\x8F\x16\x91P\x7F\xD1\xEC:\x12\x16\xF0\x8Bn\xFFr\xE1i\xCE\xB5H\xB7\x82\xDB\x18\xA6aHRa\x8D\x86\xBB\x19\xF3\xF9\xB0\xD3\x90``\x01`@Q\x80\x91\x03\x90\xA3`\x02`\x07\x84\x01T`\x01`\x80\x1B\x90\x04`\xFF\x16`\x02\x81\x11\x15a\x10\x07Wa\x10\x07a@\x19V[\x03a\x10kW\x82T`@Qc\x9E\xE4\xAF\xA3`\xE0\x1B\x81R`\x01`\x01`\xA0\x1B\x03\x90\x91\x16\x90c\x9E\xE4\xAF\xA3\x90a\x10=\x90\x89\x90\x89\x90`\x04\x01aI\xDAV[_`@Q\x80\x83\x03\x81_\x87\x80;\x15\x80\x15a\x10TW__\xFD[PZ\xF1\x15\x80\x15a\x10fW=__>=_\xFD[PPPP[PPPa\x10va5\xADV[PPPPPPPPPV[\x7F\xABf\xE1\x1COq,\xD0j\xB1\x1B\xF93\x9BH\xBE\xF3\x9E\x12\xD4\xA2.\xEE\xF7\x1D(`\xA0\xC9\x04\x82\xBDa\x10\xAB\x81a.\x99V[c\xFF\xFF\xFF\xFF\x82\x16\x15\x80a\x10\xC9WP`~Tc\xFF\xFF\xFF\xFF\x90\x81\x16\x90\x83\x16\x11[\x15a\x10\xE7W`@Qcu\x12\xE5\xCB`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[c\xFF\xFF\xFF\xFF\x82\x16_\x90\x81R`\x7F` R`@\x90 `\x01\x81\x01T`\x01`\xE8\x1B\x90\x04`\xFF\x16\x15a\x11(W`@Qc;\x8D=\x99`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x01\x81\x01\x80T`\xFF`\xE8\x1B\x19\x16`\x01`\xE8\x1B\x17\x90U`@Qc\xFF\xFF\xFF\xFF\x84\x16\x90\x7FG\x10\xD2\xEEV~\xF1\xEDn\xB2\xF6Q\xDD\xE4X\x95$\xBC\xF7\xCE\xBCb\x14z\x99\xB2\x81\xCC\x83n~D\x90_\x90\xA2PPPV[a\x11\xD5`@\x80Qa\x01\x80\x81\x01\x82R_\x80\x82R` \x82\x01\x81\x90R\x91\x81\x01\x82\x90R``\x81\x01\x82\x90R`\x80\x81\x01\x82\x90R`\xA0\x81\x01\x82\x90R`\xC0\x81\x01\x82\x90R`\xE0\x81\x01\x82\x90Ra\x01\0\x81\x01\x82\x90R\x90a\x01 \x82\x01\x90\x81R` \x01_\x81R` \x01_\x81RP\x90V[c\xFF\xFF\xFF\xFF\x82\x16_\x90\x81R`\x81` \x90\x81R`@\x91\x82\x90 \x80T`\x01`\x01`\xA0\x1B\x03\x80\x82\x16\x86R`\x01`\x01`@\x1B\x03`\x01`\xA0\x1B\x92\x83\x90\x04\x81\x16\x94\x87\x01\x94\x90\x94R`\x01\x83\x01T\x90\x81\x16\x94\x86\x01\x94\x90\x94R\x90\x92\x04\x81\x16``\x84\x01R`\x05\x82\x01T`\x80\x84\x01R`\x06\x82\x01T\x80\x82\x16`\xA0\x85\x01R`\x01`@\x1B\x90\x81\x90\x04\x82\x16`\xC0\x85\x01R`\x07\x83\x01T\x80\x83\x16`\xE0\x86\x01R\x90\x81\x04\x90\x91\x16a\x01\0\x84\x01Ra\x01 \x83\x01\x90`\xFF`\x01`\x80\x1B\x90\x91\x04\x16`\x02\x81\x11\x15a\x12\x91Wa\x12\x91a@\x19V[\x90\x81`\x02\x81\x11\x15a\x12\xA4Wa\x12\xA4a@\x19V[\x90RP`\x08\x81\x01Ta\x01@\x83\x01R`\t\x01Ta\x01`\x82\x01R\x91\x90PV[c\xFF\xFF\xFF\xFF\x86\x16_\x90\x81R`\x81` R`@\x90 ``\x90a\x12\xE6\x90\x87\x87\x87\x87\x87a5\xD7V[\x97\x96PPPPPPPV[_T`\x04\x90a\x01\0\x90\x04`\xFF\x16\x15\x80\x15a\x13\x11WP_T`\xFF\x80\x83\x16\x91\x16\x10[a\x13\x88W`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`.`$\x82\x01R\x7FInitializable: contract is alrea`D\x82\x01R\x7Fdy initialized\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`d\x82\x01R`\x84\x01[`@Q\x80\x91\x03\x90\xFD[_\x80Ta\xFF\xFF\x19\x16`\xFF\x83\x16\x17a\x01\0\x17\x90U`@\x80Q\x80\x82\x01\x82R`\t\x81Rh\x06\x16\xC2\xD7c\x02\xE32\xE3`\xBC\x1B` \x82\x01R\x90Q\x7FP\xCA\xDC\x0C\0\x1F\x05\xDDK\x81\xDB\x1E\x92\xB9\x8Dw\xE7\x18\xFD/\x10?\xB7\xB7r\x93\xE8g\xD3)\xA4\xC2\x91a\x13\xE7\x91aC\x1AV[`@Q\x80\x91\x03\x90\xA1_\x80Ta\xFF\0\x19\x16\x90U`@Q`\xFF\x82\x16\x81R\x7F\x7F&\xB8?\xF9n\x1F+jh/\x138R\xF6y\x8A\t\xC4e\xDA\x95\x92\x14`\xCE\xFB8G@$\x98\x90` \x01`@Q\x80\x91\x03\x90\xA1PV[3_\x90\x81R\x7F\xF1OZ\x8A\xD5\x9D\x90Y6\x02\xE9\x05\xB3X\"\x9B\xFF\\\xEE\xA6w\xD5\xBF\x0FZ\x17\x01y5P\xA9\xA6` R`@\x90 T`\xFF\x16\x15\x80\x15a\x14\xE1WP3`\x01`\x01`\xA0\x1B\x03\x16\x82`\x01`\x01`\xA0\x1B\x03\x16c\xF8Q\xA4@`@Q\x81c\xFF\xFF\xFF\xFF\x16`\xE0\x1B\x81R`\x04\x01` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a\x14\xB1W=__>=_\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\x14\xD5\x91\x90aI\xEDV[`\x01`\x01`\xA0\x1B\x03\x16\x14\x15[\x15a\x14\xFFW`@Qc\r\x03h\x7F`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x01`\x01`\xA0\x1B\x03\x82\x16_\x90\x81R`\x82` R`@\x81 Tc\xFF\xFF\xFF\xFF\x16\x90\x81\x90\x03a\x15>W`@Qct\xA0\x86\xA3`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[c\xFF\xFF\xFF\xFF\x81\x16_\x90\x81R`\x81` R`@\x81 \x90`\x07\x82\x01T`\x01`\x80\x1B\x90\x04`\xFF\x16`\x02\x81\x11\x15a\x15sWa\x15sa@\x19V[\x14a\x15\x91W`@Qc\x90\xDB\r\x07`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x06\x81\x01T`\x01`\x01`@\x1B\x03\x90\x81\x16\x90\x84\x16\x81\x11\x15\x80a\x15\xC9WP`\x06\x82\x01T`\x01`\x01`@\x1B\x03`\x01`@\x1B\x90\x91\x04\x81\x16\x90\x85\x16\x10[\x15a\x15\xE7W`@Qc\xCB#\xEB\xDF`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x80[\x84`\x01`\x01`@\x1B\x03\x16\x81`\x01`\x01`@\x1B\x03\x16\x14a\x16\x88W`\x01`\x01`@\x1B\x03\x80\x82\x16_\x90\x81R`\x03\x85\x01` R`@\x90 `\x01\x01T`\x01`@\x1B\x90\x04\x81\x16\x90\x86\x16\x81\x10\x15a\x16LW`@Qc\x97S\x96_`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x01`\x01`@\x1B\x03\x90\x91\x16_\x90\x81R`\x03\x84\x01` R`@\x81 \x90\x81U`\x01\x01\x80To\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x19\x16\x90Ua\x15\xE9V[`\x06\x83\x01\x80Tg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x19\x16`\x01`\x01`@\x1B\x03\x87\x16\x17\x90Ua\x16\xB0\x85\x83aJ\x08V[`\x84\x80T_\x90a\x16\xCA\x90\x84\x90`\x01`\x01`@\x1B\x03\x16aJ\x08V[\x82Ta\x01\0\x92\x90\x92\n`\x01`\x01`@\x1B\x03\x81\x81\x02\x19\x90\x93\x16\x91\x83\x16\x02\x17\x90\x91U\x86\x16_\x81\x81R`\x03\x86\x01` R`@\x90\x81\x90 T\x90Qc3Mog`\xE1\x1B\x81R`\x04\x81\x01\x92\x90\x92R`$\x82\x01R`\x01`\x01`\xA0\x1B\x03\x88\x16\x91Pcf\x9A\xDE\xCE\x90`D\x01_`@Q\x80\x83\x03\x81_\x87\x80;\x15\x80\x15a\x17CW__\xFD[PZ\xF1\x15\x80\x15a\x17UW=__>=_\xFD[PPPP`\x01`\x01`@\x1B\x03\x85\x16_\x81\x81R`\x03\x85\x01` \x90\x81R`@\x91\x82\x90 T\x91Q\x91\x82Rc\xFF\xFF\xFF\xFF\x87\x16\x91\x7F\x80\xA6\xD3\x95\xA5Z\xED\x81&\x07\x9C\xB8$\x7F\nhH\xB1D\x0C\xA2\xCD\xCA;C\x86\xF2P\xC3R\x94\x02\x91\x01`@Q\x80\x91\x03\x90\xA3PPPPPPV[`@Qcp\xA0\x821`\xE0\x1B\x81R0`\x04\x82\x01R_\x90\x81\x90`\x01`\x01`\xA0\x1B\x03\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x16\x90cp\xA0\x821\x90`$\x01` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a\x18\x1DW=__>=_\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a\x18A\x91\x90aIAV[`\x84T\x90\x91P_\x90a\x18e\x90`\x01`\x01`@\x1B\x03`\x01`@\x1B\x82\x04\x81\x16\x91\x16aJ\x08V[`\x01`\x01`@\x1B\x03\x16\x90P\x80_\x03a\x18\x7FW_\x92PPP\x90V[a\x18\x89\x81\x83aJ;V[\x92PPP\x90V[`oT_\x90`\xFF\x16\x15a\x18\xB6W`@Qc\x0B\xC0\x11\xFF`\xE2\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[3_\x90\x81R`\x82` R`@\x81 Tc\xFF\xFF\xFF\xFF\x16\x90\x81\x90\x03a\x18\xECW`@Qcqe<\x15`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x83`\x01`\x01`@\x1B\x03\x16_\x03a\x19\x15W`@Qc%\x90\xCC\xF9`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[c\xFF\xFF\xFF\xFF\x81\x16_\x90\x81R`\x81` R`@\x81 \x90`\x07\x82\x01T`\x01`\x80\x1B\x90\x04`\xFF\x16`\x02\x81\x11\x15a\x19JWa\x19Ja@\x19V[\x14a\x19hW`@Qc\x90\xDB\r\x07`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x84\x80T\x86\x91\x90_\x90a\x19\x85\x90\x84\x90`\x01`\x01`@\x1B\x03\x16aI\x0BV[\x82Ta\x01\0\x92\x90\x92\n`\x01`\x01`@\x1B\x03\x81\x81\x02\x19\x90\x93\x16\x91\x83\x16\x02\x17\x90\x91U`\x06\x83\x01T\x16\x90P_a\x19\xB8\x87\x83aI\x0BV[`\x06\x84\x01\x80T`\x01`\x01`@\x1B\x03\x83\x81\x16g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x19\x90\x92\x16\x82\x17\x90\x92U`@\x80Q``\x81\x01\x82R\x8A\x81RB\x84\x16` \x80\x83\x01\x91\x82R\x88\x86\x16\x83\x85\x01\x90\x81R_\x86\x81R`\x03\x8C\x01\x83R\x85\x90 \x93Q\x84U\x91Q`\x01\x93\x90\x93\x01\x80T\x92Q\x87\x16`\x01`@\x1B\x02o\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x19\x90\x93\x16\x93\x90\x96\x16\x92\x90\x92\x17\x17\x90\x93UQ\x90\x81R\x91\x92Pc\xFF\xFF\xFF\xFF\x86\x16\x91\x7F\x1D\x9F0&\0Q\xD5\x1Dp3\x9D\xA29\xEA{\x08\0!\xAD\xCA\xAB\xFAq\xC9\xB0\xEA3\x9A \xCF\x9A%\x91\x01`@Q\x80\x91\x03\x90\xA2\x96\x95PPPPPPV[`\x80T_\x90c\xFF\xFF\xFF\xFF\x16\x80\x82\x03a\x1A\x9FWP_\x91\x90PV[_\x81`\x01`\x01`@\x1B\x03\x81\x11\x15a\x1A\xB8Wa\x1A\xB8aC\x97V[`@Q\x90\x80\x82R\x80` \x02` \x01\x82\x01`@R\x80\x15a\x1A\xE1W\x81` \x01` \x82\x02\x806\x837\x01\x90P[P\x90P_[\x82\x81\x10\x15a\x1B>W`\x81_a\x1A\xFC\x83`\x01aJNV[c\xFF\xFF\xFF\xFF\x16c\xFF\xFF\xFF\xFF\x16\x81R` \x01\x90\x81R` \x01_ `\x05\x01T\x82\x82\x81Q\x81\x10a\x1B+Wa\x1B+aJaV[` \x90\x81\x02\x91\x90\x91\x01\x01R`\x01\x01a\x1A\xE6V[P_` [\x83`\x01\x14a\x1DYW_a\x1BW`\x02\x86aJuV[a\x1Bb`\x02\x87aJ;V[a\x1Bl\x91\x90aJNV[\x90P_\x81`\x01`\x01`@\x1B\x03\x81\x11\x15a\x1B\x87Wa\x1B\x87aC\x97V[`@Q\x90\x80\x82R\x80` \x02` \x01\x82\x01`@R\x80\x15a\x1B\xB0W\x81` \x01` \x82\x02\x806\x837\x01\x90P[P\x90P_[\x82\x81\x10\x15a\x1D\tWa\x1B\xC8`\x01\x84aJ\x88V[\x81\x14\x80\x15a\x1B\xE0WPa\x1B\xDC`\x02\x88aJuV[`\x01\x14[\x15a\x1C]W\x85a\x1B\xF1\x82`\x02aI*V[\x81Q\x81\x10a\x1C\x01Wa\x1C\x01aJaV[` \x02` \x01\x01Q\x85`@Q` \x01a\x1C$\x92\x91\x90\x91\x82R` \x82\x01R`@\x01\x90V[`@Q` \x81\x83\x03\x03\x81R\x90`@R\x80Q\x90` \x01 \x82\x82\x81Q\x81\x10a\x1CLWa\x1CLaJaV[` \x02` \x01\x01\x81\x81RPPa\x1D\x01V[\x85a\x1Ci\x82`\x02aI*V[\x81Q\x81\x10a\x1CyWa\x1CyaJaV[` \x02` \x01\x01Q\x86\x82`\x02a\x1C\x8F\x91\x90aI*V[a\x1C\x9A\x90`\x01aJNV[\x81Q\x81\x10a\x1C\xAAWa\x1C\xAAaJaV[` \x02` \x01\x01Q`@Q` \x01a\x1C\xCC\x92\x91\x90\x91\x82R` \x82\x01R`@\x01\x90V[`@Q` \x81\x83\x03\x03\x81R\x90`@R\x80Q\x90` \x01 \x82\x82\x81Q\x81\x10a\x1C\xF4Wa\x1C\xF4aJaV[` \x02` \x01\x01\x81\x81RPP[`\x01\x01a\x1B\xB5V[P\x80\x94P\x81\x95P\x83\x84`@Q` \x01a\x1D,\x92\x91\x90\x91\x82R` \x82\x01R`@\x01\x90V[`@Q` \x81\x83\x03\x03\x81R\x90`@R\x80Q\x90` \x01 \x93P\x82\x80a\x1DO\x90aJ\x9BV[\x93PPPPa\x1BCV[_\x83_\x81Q\x81\x10a\x1DlWa\x1DlaJaV[` \x02` \x01\x01Q\x90P__\x90P[\x82\x81\x10\x15a\x1D\xE2W`@\x80Q` \x81\x01\x84\x90R\x90\x81\x01\x85\x90R``\x01`@\x80Q`\x1F\x19\x81\x84\x03\x01\x81R\x82\x82R\x80Q` \x91\x82\x01 \x90\x83\x01\x87\x90R\x90\x82\x01\x86\x90R\x92P``\x01`@\x80Q`\x1F\x19\x81\x84\x03\x01\x81R\x91\x90R\x80Q` \x90\x91\x01 \x93P`\x01\x01a\x1D{V[P\x95\x94PPPPPV[\x7F\xACu\xD2M\xBB5\xEA\x80\xE2_\xAB\x16}\xA4\xDE\xA4l\x19\x15&\x04&W\r\xB8O\x18H\x91\xF5\xF5\x90a\x1E\x16\x81a.\x99V[`~\x80T_\x91\x90\x82\x90a\x1E.\x90c\xFF\xFF\xFF\xFF\x16aJ\xB0V[\x91\x90a\x01\0\n\x81T\x81c\xFF\xFF\xFF\xFF\x02\x19\x16\x90\x83c\xFF\xFF\xFF\xFF\x16\x02\x17\x90U\x90P`\x01`\x02\x81\x11\x15a\x1E`Wa\x1E`a@\x19V[\x86`\x02\x81\x11\x15a\x1ErWa\x1Era@\x19V[\x03a\x1E\x9BW\x84\x15a\x1E\x96W`@Qcc\xD7\"\xE7`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\x1FUV[`\x02\x86`\x02\x81\x11\x15a\x1E\xAFWa\x1E\xAFa@\x19V[\x03a\x1F\x05W`\x01`\x01`\xA0\x1B\x03\x88\x16\x15\x15\x80a\x1E\xD3WP`\x01`\x01`@\x1B\x03\x87\x16\x15\x15[\x80a\x1E\xDDWP\x84\x15\x15[\x80a\x1E\xE7WP\x82\x15\x15[\x15a\x1E\x96W`@Qcc\xD7\"\xE7`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[_\x86`\x02\x81\x11\x15a\x1F\x18Wa\x1F\x18a@\x19V[\x03a\x1F<W\x82\x15a\x1E\x96W`@Qcc\xD7\"\xE7`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`@Qcc\xD7\"\xE7`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`@Q\x80`\xE0\x01`@R\x80\x8A`\x01`\x01`\xA0\x1B\x03\x16\x81R` \x01\x89`\x01`\x01`\xA0\x1B\x03\x16\x81R` \x01\x88`\x01`\x01`@\x1B\x03\x16\x81R` \x01\x87`\x02\x81\x11\x15a\x1F\x9FWa\x1F\x9Fa@\x19V[\x81R_` \x80\x83\x01\x82\x90R`@\x80\x84\x01\x8A\x90R``\x93\x84\x01\x88\x90Rc\xFF\xFF\xFF\xFF\x86\x16\x83R`\x7F\x82R\x91\x82\x90 \x84Q\x81T`\x01`\x01`\xA0\x1B\x03\x91\x82\x16s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x19\x90\x91\x16\x17\x82U\x91\x85\x01Q`\x01\x82\x01\x80T\x94\x87\x01Q`\x01`\x01`@\x1B\x03\x16`\x01`\xA0\x1B\x02`\x01`\x01`\xE0\x1B\x03\x19\x90\x95\x16\x91\x90\x93\x16\x17\x92\x90\x92\x17\x80\x82U\x92\x84\x01Q\x91\x92`\xFF`\xE0\x1B\x19\x16`\x01`\xE0\x1B\x83`\x02\x81\x11\x15a OWa Oa@\x19V[\x02\x17\x90UP`\x80\x82\x01Q`\x01\x82\x01\x80T\x91\x15\x15`\x01`\xE8\x1B\x02`\xFF`\xE8\x1B\x19\x90\x92\x16\x91\x90\x91\x17\x90U`\xA0\x82\x01Q`\x02\x82\x01U`\xC0\x90\x91\x01Q`\x03\x90\x91\x01U`@Qc\xFF\xFF\xFF\xFF\x82\x16\x90\x7F\x9E\xAF.\xCB\xDD\xB1H\x89\xC9\xE1A\xA61u\xC5Z\xC2^\x0C\xD7\xCD\xEA1,\xDF\xBD\x03\x97\x97k8:\x90a \xD1\x90\x8C\x90\x8C\x90\x8C\x90\x8C\x90\x8C\x90\x8C\x90\x8C\x90aJ\xD4V[`@Q\x80\x91\x03\x90\xA2PPPPPPPPPV[\x7Ff\x15f\x03\xFE)\xD1?\x97\xC6\xF3\xE3\xDF\xF4\xEFq\x91\x9F\x9A\xA6\x1CU[\xE0\x18-\x95N\x94\"\x1A\xACa!\x0E\x81a.\x99V[a!\x19\x84\x84\x84a6\xDAV[PPPPV[\x7F\xA0\xFA\xB0t\xAB\xA3jo\xA6\x9F\x1A\x83\xEE\x86\xE5\xAB\xFB\x843\x96n\xB5~\xFB\x13\xDC/\xC2\xF2M\xDD\x08a!I\x81a.\x99V[c\xFF\xFF\xFF\xFF\x89\x16\x15\x80a!gWP`~Tc\xFF\xFF\xFF\xFF\x90\x81\x16\x90\x8A\x16\x11[\x15a!\x85W`@Qcu\x12\xE5\xCB`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[c\xFF\xFF\xFF\xFF\x89\x16_\x90\x81R`\x7F` R`@\x90 `\x01\x81\x01T`\x01`\xE8\x1B\x90\x04`\xFF\x16\x15a!\xC6W`@Qc;\x8D=\x99`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[c\xFF\xFF\xFF\xFF`\x01`\x01`@\x1B\x03\x8A\x16\x11\x15a!\xF4W`@QcLu?W`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x01`\x01`@\x1B\x03\x89\x16_\x90\x81R`\x83` R`@\x90 Tc\xFF\xFF\xFF\xFF\x16\x15a\"0W`@Qc7\xC8\xFE\t`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x80\x80T_\x91\x90\x82\x90a\"H\x90c\xFF\xFF\xFF\xFF\x16aJ\xB0V[\x82Tc\xFF\xFF\xFF\xFF\x82\x81\x16a\x01\0\x94\x90\x94\n\x93\x84\x02\x93\x02\x19\x16\x91\x90\x91\x17\x90\x91U\x82T`@\x80Q_\x80\x82R` \x82\x01\x92\x83\x90R\x93\x94P`\x01`\x01`\xA0\x1B\x03\x90\x92\x16\x910\x91a\"\x93\x90a>\xA0V[a\"\x9F\x93\x92\x91\x90aK:V[`@Q\x80\x91\x03\x90_\xF0\x80\x15\x80\x15a\"\xB8W=__>=_\xFD[P\x90P\x81`\x83_\x8D`\x01`\x01`@\x1B\x03\x16`\x01`\x01`@\x1B\x03\x16\x81R` \x01\x90\x81R` \x01_ _a\x01\0\n\x81T\x81c\xFF\xFF\xFF\xFF\x02\x19\x16\x90\x83c\xFF\xFF\xFF\xFF\x16\x02\x17\x90UP\x81`\x82_\x83`\x01`\x01`\xA0\x1B\x03\x16`\x01`\x01`\xA0\x1B\x03\x16\x81R` \x01\x90\x81R` \x01_ _a\x01\0\n\x81T\x81c\xFF\xFF\xFF\xFF\x02\x19\x16\x90\x83c\xFF\xFF\xFF\xFF\x16\x02\x17\x90UP_`\x81_\x84c\xFF\xFF\xFF\xFF\x16c\xFF\xFF\xFF\xFF\x16\x81R` \x01\x90\x81R` \x01_ \x90P\x81\x81_\x01_a\x01\0\n\x81T\x81`\x01`\x01`\xA0\x1B\x03\x02\x19\x16\x90\x83`\x01`\x01`\xA0\x1B\x03\x16\x02\x17\x90UP\x83`\x01\x01`\x14\x90T\x90a\x01\0\n\x90\x04`\x01`\x01`@\x1B\x03\x16\x81`\x01\x01`\x14a\x01\0\n\x81T\x81`\x01`\x01`@\x1B\x03\x02\x19\x16\x90\x83`\x01`\x01`@\x1B\x03\x16\x02\x17\x90UP\x83`\x01\x01_\x90T\x90a\x01\0\n\x90\x04`\x01`\x01`\xA0\x1B\x03\x16\x81`\x01\x01_a\x01\0\n\x81T\x81`\x01`\x01`\xA0\x1B\x03\x02\x19\x16\x90\x83`\x01`\x01`\xA0\x1B\x03\x16\x02\x17\x90UP\x8B\x81_\x01`\x14a\x01\0\n\x81T\x81`\x01`\x01`@\x1B\x03\x02\x19\x16\x90\x83`\x01`\x01`@\x1B\x03\x16\x02\x17\x90UP\x83`\x02\x01T\x81`\x02\x01__`\x01`\x01`@\x1B\x03\x16\x81R` \x01\x90\x81R` \x01_ \x81\x90UP\x8Cc\xFF\xFF\xFF\xFF\x16\x81`\x07\x01`\x08a\x01\0\n\x81T\x81`\x01`\x01`@\x1B\x03\x02\x19\x16\x90\x83`\x01`\x01`@\x1B\x03\x16\x02\x17\x90UP\x83`\x01\x01`\x1C\x90T\x90a\x01\0\n\x90\x04`\xFF\x16\x81`\x07\x01`\x10a\x01\0\n\x81T\x81`\xFF\x02\x19\x16\x90\x83`\x02\x81\x11\x15a$\xB5Wa$\xB5a@\x19V[\x02\x17\x90UP\x83`\x03\x01T\x81`\t\x01\x81\x90UP\x82c\xFF\xFF\xFF\xFF\x16\x7F\x19L\x984V\xDFg\x01\xC6\xA5\x080\xB9\x0F\xE8\x0Er\xB8#A\x1D\rRIp\xC9Y\r\xC2w\xA6A\x8E\x84\x8F\x8D`@Qa%6\x94\x93\x92\x91\x90c\xFF\xFF\xFF\xFF\x94\x90\x94\x16\x84R`\x01`\x01`\xA0\x1B\x03\x92\x83\x16` \x85\x01R`\x01`\x01`@\x1B\x03\x91\x90\x91\x16`@\x84\x01R\x16``\x82\x01R`\x80\x01\x90V[`@Q\x80\x91\x03\x90\xA2`\x02`\x01\x85\x01T`\x01`\xE0\x1B\x90\x04`\xFF\x16`\x02\x81\x11\x15a%`Wa%`a@\x19V[\x03a%\xC3W`@QcC\x9F\xAB\x91`\xE0\x1B\x81R`\x01`\x01`\xA0\x1B\x03\x83\x16\x90cC\x9F\xAB\x91\x90a%\x91\x90\x89\x90`\x04\x01aC\x1AV[_`@Q\x80\x83\x03\x81_\x87\x80;\x15\x80\x15a%\xA8W__\xFD[PZ\xF1\x15\x80\x15a%\xBAW=__>=_\xFD[PPPPa&'V[`@Qc8\x92\xB8\x11`\xE1\x1B\x81R`\x01`\x01`\xA0\x1B\x03\x83\x16\x90cq%p\"\x90a%\xF9\x90\x8E\x90\x8E\x90\x88\x90\x8F\x90\x8F\x90\x8F\x90`\x04\x01aKsV[_`@Q\x80\x83\x03\x81_\x87\x80;\x15\x80\x15a&\x10W__\xFD[PZ\xF1\x15\x80\x15a&\"W=__>=_\xFD[PPPP[PPPPPPPPPPPPPV[\x7F\x8C\xF8\x07\xF6\x97\x07 \xF8\xE2\xC2\x08\xC7\xC5\x03u\x95\x98,{\xD9\xED\x93\xC3\x80\xD0\x9D\xF7C\xD0\xDC\xC3\xFBa&`\x81a.\x99V[h65\xC9\xAD\xC5\xDE\xA0\0\0\x82\x11\x80a&zWPc;\x9A\xCA\0\x82\x10[\x15a&\x98W`@Qc\x85\x86\x95%`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x86\x82\x90U`@Q\x82\x81R\x7F\xFB86S\xF5>\xE0y\x97\x8D\x0C\x9A\xFFz\xEF\xF0J\x10\x16l\xE2D\xCC\xA9\xC9\xF9\xD8\xD9k\xEDE\xB2\x90` \x01`@Q\x80\x91\x03\x90\xA1PPV[_\x82\x81R`4` R`@\x90 `\x01\x01Ta&\xEE\x81a.\x99V[a\x0C@\x83\x83a2\xEAV[\x7Fb\xBAk\xA2\xFF\xED\x8C\xFE1kX3%\xEAA\xACn{\xA9\xE5\x86M+\xC6\xFA\xBB\xA7\xAC&\xD2\xF0\xF4a'\"\x81a.\x99V[`\x87\x80Tg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x19\x16B`\x01`\x01`@\x1B\x03\x16\x17\x90U`@\x80Qcm\xE0\xB4\xBB`\xE1\x1B\x81R\x90Q\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`\x01`\x01`\xA0\x1B\x03\x16\x91c\xDB\xC1iv\x91`\x04\x80\x83\x01\x92_\x92\x91\x90\x82\x90\x03\x01\x81\x83\x87\x80;\x15\x80\x15a'\x9DW__\xFD[PZ\xF1\x15\x80\x15a'\xAFW=__>=_\xFD[PPPPa'\xBBa:nV[PV[c\xFF\xFF\xFF\xFF\x86\x16_\x90\x81R`\x81` R`@\x90 ``\x90a\x12\xE6\x90\x88\x90\x88\x88\x88\x88\x88a3\xD8V[3`\x01`\x01`\xA0\x1B\x03\x16\x82`\x01`\x01`\xA0\x1B\x03\x16c\xF8Q\xA4@`@Q\x81c\xFF\xFF\xFF\xFF\x16`\xE0\x1B\x81R`\x04\x01` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a(+W=__>=_\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a(O\x91\x90aI\xEDV[`\x01`\x01`\xA0\x1B\x03\x16\x14a(vW`@Qci`r\xE9`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x01`\x01`\xA0\x1B\x03\x82\x16_\x90\x81R`\x82` \x90\x81R`@\x80\x83 Tc\xFF\xFF\xFF\xFF\x16\x83R`\x81\x90\x91R\x90 `\x06\x81\x01T`\x01`\x01`@\x1B\x03\x80\x82\x16`\x01`@\x1B\x90\x92\x04\x16\x14a(\xD7W`@QcfC\x16\xA5`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x02c\xFF\xFF\xFF\xFF\x83\x16_\x90\x81R`\x7F` R`@\x90 `\x01\x01T`\x01`\xE0\x1B\x90\x04`\xFF\x16`\x02\x81\x11\x15a)\x0CWa)\x0Ca@\x19V[\x14\x15\x80\x15a)5WP`\x07\x81\x01Tc\xFF\xFF\xFF\xFF\x83\x16`\x01`@\x1B\x90\x91\x04`\x01`\x01`@\x1B\x03\x16\x10\x15[\x15a)SW`@Qc>7\xE23`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x01`\x01`\xA0\x1B\x03\x83\x16_\x90\x81R`\x82` \x90\x81R`@\x80\x83 Tc\xFF\xFF\xFF\xFF\x86\x81\x16\x85R`\x7F\x90\x93R\x92 `\x01\x01T\x91\x16\x90`\x01`\xE0\x1B\x90\x04`\xFF\x16`\x02\x81\x11\x15a)\xA1Wa)\xA1a@\x19V[c\xFF\xFF\xFF\xFF\x82\x16_\x90\x81R`\x7F` R`@\x90 `\x01\x01T`\x01`\xE0\x1B\x90\x04`\xFF\x16`\x02\x81\x11\x15a)\xD4Wa)\xD4a@\x19V[\x14a)\xF2W`@QcZ\xA0\xD5\xF1`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`@\x80Q_\x81R` \x81\x01\x90\x91Ra!\x19\x90\x85\x90\x85\x90a6\xDAV[\x7F=\xFE'}*,\x04\xB7_\xB2\xEB7C\xFA\0\0Z\xE3g\x8A \xC2\x99\xE6_\xDFM\xF7e\x17\xF6\x8Ea*7\x81a.\x99V[`\x01`\x01`@\x1B\x03\x86\x16_\x90\x81R`\x83` R`@\x90 Tc\xFF\xFF\xFF\xFF\x16\x15a*sW`@Qc7\xC8\xFE\t`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[c\xFF\xFF\xFF\xFF`\x01`\x01`@\x1B\x03\x87\x16\x11\x15a*\xA1W`@QcLu?W`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x01`\x01`\xA0\x1B\x03\x89\x16_\x90\x81R`\x82` R`@\x90 Tc\xFF\xFF\xFF\xFF\x16\x15a*\xDDW`@Qc\r@\x9B\x93`\xE4\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x80\x80T_\x91\x90\x82\x90a*\xF5\x90c\xFF\xFF\xFF\xFF\x16aJ\xB0V[\x91\x90a\x01\0\n\x81T\x81c\xFF\xFF\xFF\xFF\x02\x19\x16\x90\x83c\xFF\xFF\xFF\xFF\x16\x02\x17\x90U\x90P\x80`\x83_\x89`\x01`\x01`@\x1B\x03\x16`\x01`\x01`@\x1B\x03\x16\x81R` \x01\x90\x81R` \x01_ _a\x01\0\n\x81T\x81c\xFF\xFF\xFF\xFF\x02\x19\x16\x90\x83c\xFF\xFF\xFF\xFF\x16\x02\x17\x90UP\x80`\x82_\x8C`\x01`\x01`\xA0\x1B\x03\x16`\x01`\x01`\xA0\x1B\x03\x16\x81R` \x01\x90\x81R` \x01_ _a\x01\0\n\x81T\x81c\xFF\xFF\xFF\xFF\x02\x19\x16\x90\x83c\xFF\xFF\xFF\xFF\x16\x02\x17\x90UP_`\x81_\x83c\xFF\xFF\xFF\xFF\x16c\xFF\xFF\xFF\xFF\x16\x81R` \x01\x90\x81R` \x01_ \x90P\x8A\x81_\x01_a\x01\0\n\x81T\x81`\x01`\x01`\xA0\x1B\x03\x02\x19\x16\x90\x83`\x01`\x01`\xA0\x1B\x03\x16\x02\x17\x90UP\x88\x81`\x01\x01`\x14a\x01\0\n\x81T\x81`\x01`\x01`@\x1B\x03\x02\x19\x16\x90\x83`\x01`\x01`@\x1B\x03\x16\x02\x17\x90UP\x89\x81`\x01\x01_a\x01\0\n\x81T\x81`\x01`\x01`\xA0\x1B\x03\x02\x19\x16\x90\x83`\x01`\x01`\xA0\x1B\x03\x16\x02\x17\x90UP\x87\x81_\x01`\x14a\x01\0\n\x81T\x81`\x01`\x01`@\x1B\x03\x02\x19\x16\x90\x83`\x01`\x01`@\x1B\x03\x16\x02\x17\x90UP\x85\x81`\x07\x01`\x10a\x01\0\n\x81T\x81`\xFF\x02\x19\x16\x90\x83`\x02\x81\x11\x15a,|Wa,|a@\x19V[\x02\x17\x90UP`\x01\x86`\x02\x81\x11\x15a,\x95Wa,\x95a@\x19V[\x03a,\xADW`\t\x81\x01\x85\x90U`\x05\x81\x01\x87\x90Ua,\xECV[`\x02\x86`\x02\x81\x11\x15a,\xC1Wa,\xC1a@\x19V[\x03a,\xD9W`\x08\x81\x01\x84\x90U`\x05\x81\x01\x87\x90Ua,\xECV[_\x80\x80R`\x02\x82\x01` R`@\x90 \x87\x90U[\x81c\xFF\xFF\xFF\xFF\x16\x7FM\xA4\x7Fn\x9B\xBD\x9E\xF9\x18\x87\x18:Wj\xAE\xBC\xF1\xB9\xBB}*V{3\xB0u\x04Lm6\x08.\x8A\x8D\x8B\x8A_\x8B\x8B`@Qa-.\x97\x96\x95\x94\x93\x92\x91\x90aK\xDDV[`@Q\x80\x91\x03\x90\xA2PPPPPPPPPPPV[a-\xA7`@\x80Qa\x01\x80\x81\x01\x82R_\x80\x82R` \x82\x01\x81\x90R\x91\x81\x01\x82\x90R``\x81\x01\x82\x90R`\x80\x81\x01\x82\x90R`\xA0\x81\x01\x82\x90R`\xC0\x81\x01\x82\x90R`\xE0\x81\x01\x82\x90Ra\x01\0\x81\x01\x82\x90Ra\x01 \x81\x01\x82\x90Ra\x01@\x81\x01\x82\x90R\x90a\x01`\x82\x01R\x90V[c\xFF\xFF\xFF\xFF\x82\x16_\x90\x81R`\x81` \x90\x81R`@\x91\x82\x90 \x80T`\x01`\x01`\xA0\x1B\x03\x80\x82\x16\x86R`\x01`\xA0\x1B\x91\x82\x90\x04`\x01`\x01`@\x1B\x03\x90\x81\x16\x94\x87\x01\x94\x90\x94R`\x01\x83\x01T\x90\x81\x16\x94\x86\x01\x94\x90\x94R\x90\x92\x04\x81\x16``\x84\x01R`\x05\x82\x01T`\x80\x84\x01R`\x06\x82\x01T\x80\x82\x16`\xA0\x85\x01R`\x01`@\x1B\x80\x82\x04\x83\x16`\xC0\x86\x01R`\x01`\x80\x1B\x80\x83\x04\x84\x16`\xE0\x87\x01R`\x01`\xC0\x1B\x90\x92\x04\x83\x16a\x01\0\x86\x01R`\x07\x84\x01T\x80\x84\x16a\x01 \x87\x01R\x90\x81\x04\x90\x92\x16a\x01@\x85\x01Ra\x01`\x84\x01\x91\x04`\xFF\x16`\x02\x81\x11\x15a.|Wa.|a@\x19V[\x90\x81`\x02\x81\x11\x15a.\x8FWa.\x8Fa@\x19V[\x81RPPP\x91\x90PV[a'\xBB\x813a:\xC5V[__a.\xC1\x89`\x06\x01T`\x01`\x01`@\x1B\x03`\x01`@\x1B\x90\x91\x04\x16\x90V[`\x07\x8A\x01T\x90\x91P`\x01`\x01`@\x1B\x03\x90\x81\x16\x90\x89\x16\x10\x15a.\xF6W`@Qc\xEA\xD14\x0B`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x01`\x01`@\x1B\x03\x88\x16_\x90\x81R`\x02\x8A\x01` R`@\x90 T\x91P\x81a/0W`@Qc$\xCB\xDC\xC3`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x80`\x01`\x01`@\x1B\x03\x16\x88`\x01`\x01`@\x1B\x03\x16\x11\x15a/cW`@Qc\x0F+t\xF1`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x80`\x01`\x01`@\x1B\x03\x16\x87`\x01`\x01`@\x1B\x03\x16\x11a/\x95W`@Qc\xB9\xB1\x8FW`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[_a/\xA4\x8A\x8A\x8A\x8A\x87\x8Ba5\xD7V[\x90P_\x7F0dNr\xE11\xA0)\xB8PE\xB6\x81\x81X](3\xE8Hy\xB9p\x91C\xE1\xF5\x93\xF0\0\0\x01`\x02\x83`@Qa/\xD8\x91\x90aL8V[` `@Q\x80\x83\x03\x81\x85Z\xFA\x15\x80\x15a/\xF3W=__>=_\xFD[PPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a0\x16\x91\x90aIAV[a0 \x91\x90aJuV[`\x01\x8C\x01T`@\x80Q` \x81\x01\x82R\x83\x81R\x90QcH\x90\xEDE`\xE1\x1B\x81R\x92\x93P`\x01`\x01`\xA0\x1B\x03\x90\x91\x16\x91c\x91!\xDA\x8A\x91a0b\x91\x89\x91\x90`\x04\x01aLNV[` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a0}W=__>=_\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a0\xA1\x91\x90aL\x8AV[a0\xBEW`@Qc\t\xBD\xE39`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[_a0\xC9\x84\x8BaJ\x08V[\x90Pa1\x1C\x87\x82`\x01`\x01`@\x1B\x03\x16a0\xE1a\x17\xB7V[a0\xEB\x91\x90aI*V[`\x01`\x01`\xA0\x1B\x03\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x16\x91\x90a;\x07V[\x80`\x84`\x08\x82\x82\x82\x90T\x90a\x01\0\n\x90\x04`\x01`\x01`@\x1B\x03\x16a1@\x91\x90aI\x0BV[\x82Ta\x01\0\x92\x90\x92\n`\x01`\x01`@\x1B\x03\x81\x81\x02\x19\x90\x93\x16\x91\x83\x16\x02\x17\x90\x91U`\x84\x80Tg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF`\x80\x1B\x19\x16`\x01`\x80\x1BB\x84\x16\x02\x17\x90U\x8DT`@Qc2\xC2\xD1S`\xE0\x1B\x81R\x91\x8D\x16`\x04\x83\x01R`$\x82\x01\x8B\x90R3`D\x83\x01R`\x01`\x01`\xA0\x1B\x03\x16\x91Pc2\xC2\xD1S\x90`d\x01_`@Q\x80\x83\x03\x81_\x87\x80;\x15\x80\x15a1\xCDW__\xFD[PZ\xF1\x15\x80\x15a1\xDFW=__>=_\xFD[PPPPPPPPPPPPPPPPV[\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`\x01`\x01`\xA0\x1B\x03\x16c r\xF6\xC5`@Q\x81c\xFF\xFF\xFF\xFF\x16`\xE0\x1B\x81R`\x04\x01_`@Q\x80\x83\x03\x81_\x87\x80;\x15\x80\x15a2IW__\xFD[PZ\xF1\x15\x80\x15a2[W=__>=_\xFD[PPPPa\x0C\x1Aa;nV[_\x82\x81R`4` \x90\x81R`@\x80\x83 `\x01`\x01`\xA0\x1B\x03\x85\x16\x84R\x90\x91R\x90 T`\xFF\x16a\x0CxW_\x82\x81R`4` \x90\x81R`@\x80\x83 `\x01`\x01`\xA0\x1B\x03\x85\x16\x80\x85R\x92R\x80\x83 \x80T`\xFF\x19\x16`\x01\x17\x90UQ3\x92\x85\x91\x7F/\x87\x88\x11~~\xFF\x1D\x82\xE9&\xECyI\x01\xD1|x\x02JP'\t@0E@\xA73eo\r\x91\x90\xA4PPV[_\x82\x81R`4` \x90\x81R`@\x80\x83 `\x01`\x01`\xA0\x1B\x03\x85\x16\x84R\x90\x91R\x90 T`\xFF\x16\x15a\x0CxW_\x82\x81R`4` \x90\x81R`@\x80\x83 `\x01`\x01`\xA0\x1B\x03\x85\x16\x80\x85R\x92R\x80\x83 \x80T`\xFF\x19\x16\x90UQ3\x92\x85\x91\x7F\xF69\x1F\\2\xD9\xC6\x9D*G\xEAg\x0BD)t\xB595\xD1\xED\xC7\xFDd\xEB!\xE0G\xA89\x17\x1B\x91\x90\xA4PPV[\x7F\x9Bw\x9B\x17B-\r\xF9\"#\x01\x8B2\xB4\xD1\xFAF\xE0qr=h\x17\xE2Hm\0;\xEC\xC5_\0\\\x15a3\xABW`@Qc>\xE5\xAE\xB5`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\x0C\x1A`\x01\x7F\x9Bw\x9B\x17B-\r\xF9\"#\x01\x8B2\xB4\xD1\xFAF\xE0qr=h\x17\xE2Hm\0;\xEC\xC5_\0[\x90a;\xC9V[```\x02`\x07\x88\x01T`\x01`\x80\x1B\x90\x04`\xFF\x16`\x02\x81\x11\x15a3\xFCWa3\xFCa@\x19V[\x03a4\xD7W\x86T`@Qc\x1A\x95}\x9B`\xE2\x1B\x81R_\x91`\x01`\x01`\xA0\x1B\x03\x16\x90cjU\xF6l\x90a42\x90\x87\x90\x87\x90`\x04\x01aI\xDAV[` `@Q\x80\x83\x03\x81\x86Z\xFA\x15\x80\x15a4MW=__>=_\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a4q\x91\x90aIAV[`\x05\x89\x01T`\x08\x8A\x01T`@\x80Q` \x81\x01\x93\x90\x93R\x82\x01R``\x81\x01\x89\x90R`\x01`\x01`\xE0\x1B\x03\x19`\xE0\x8C\x90\x1B\x16`\x80\x82\x01R`\x84\x81\x01\x82\x90R`\xA4\x81\x01\x88\x90R`\xC4\x81\x01\x87\x90R\x90\x91P`\xE4\x01`@Q` \x81\x83\x03\x03\x81R\x90`@R\x91PPa\x12\xE6V[\x86T`@\x80Qc+G\xB7\xCD`\xE2\x1B\x81R\x90Q_\x92`\x01`\x01`\xA0\x1B\x03\x16\x91c\xAD\x1E\xDF4\x91`\x04\x80\x83\x01\x92` \x92\x91\x90\x82\x90\x03\x01\x81\x86Z\xFA\x15\x80\x15a5\x1DW=__>=_\xFD[PPPP`@Q=`\x1F\x19`\x1F\x82\x01\x16\x82\x01\x80`@RP\x81\x01\x90a5A\x91\x90aIAV[`\x05\x89\x01T`\x08\x8A\x01T`@\x80Q` \x81\x01\x93\x90\x93R\x82\x01R``\x81\x01\x89\x90R`\x01`\x01`\xE0\x1B\x03\x19`\xE0\x8C\x90\x1B\x16`\x80\x82\x01R`\x84\x81\x01\x82\x90R`\xA4\x81\x01\x88\x90R`\xC4\x81\x01\x87\x90R\x90\x91P`\xE4\x01`@Q` \x81\x83\x03\x03\x81R\x90`@R\x91PP\x97\x96PPPPPPPV[a\x0C\x1A_\x7F\x9Bw\x9B\x17B-\r\xF9\"#\x01\x8B2\xB4\xD1\xFAF\xE0qr=h\x17\xE2Hm\0;\xEC\xC5_\0a3\xD2V[`\x01`\x01`@\x1B\x03\x80\x86\x16_\x81\x81R`\x03\x89\x01` R`@\x80\x82 T\x93\x88\x16\x82R\x90 T``\x92\x91\x15\x80\x15\x90a6\x0BWP\x81\x15[\x15a6)W`@Qc4\x0CaO`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x80a6GW`@Qcf8[Q`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a6P\x84a;\xD0V[a6mW`@Qc\x05\xDA\xE4O`\xE2\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[3\x85\x83\x8A\x8C_\x01`\x14\x90T\x90a\x01\0\n\x90\x04`\x01`\x01`@\x1B\x03\x16\x8D`\x01\x01`\x14\x90T\x90a\x01\0\n\x90\x04`\x01`\x01`@\x1B\x03\x16\x89\x87\x8D\x8F`@Q` \x01a6\xBD\x9A\x99\x98\x97\x96\x95\x94\x93\x92\x91\x90aL\xA9V[`@Q` \x81\x83\x03\x03\x81R\x90`@R\x92PPP\x96\x95PPPPPPV[c\xFF\xFF\xFF\xFF\x82\x16\x15\x80a6\xF8WP`~Tc\xFF\xFF\xFF\xFF\x90\x81\x16\x90\x83\x16\x11[\x15a7\x16W`@Qcu\x12\xE5\xCB`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x01`\x01`\xA0\x1B\x03\x83\x16_\x90\x81R`\x82` R`@\x81 Tc\xFF\xFF\xFF\xFF\x16\x90\x81\x90\x03a7UW`@Qct\xA0\x86\xA3`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[c\xFF\xFF\xFF\xFF\x81\x81\x16_\x90\x81R`\x81` R`@\x90 `\x07\x81\x01T\x90\x91\x85\x16`\x01`@\x1B\x90\x91\x04`\x01`\x01`@\x1B\x03\x16\x03a7\xA2W`@QcOa\xD5\x19`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[c\xFF\xFF\xFF\xFF\x84\x16_\x90\x81R`\x7F` R`@\x90 `\x01\x81\x01T`\x01`\xE8\x1B\x90\x04`\xFF\x16\x15a7\xE3W`@Qc;\x8D=\x99`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x02`\x01\x82\x01T`\x01`\xE0\x1B\x90\x04`\xFF\x16`\x02\x81\x11\x15a8\x05Wa8\x05a@\x19V[\x14a8\xA8W`\x02`\x07\x83\x01T`\x01`\x80\x1B\x90\x04`\xFF\x16`\x02\x81\x11\x15a8,Wa8,a@\x19V[\x03a8JW`@QcZ\xA0\xD5\xF1`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x01\x81\x01T`\x01`\xE0\x1B\x90\x04`\xFF\x16`\x02\x81\x11\x15a8jWa8ja@\x19V[`\x07\x83\x01T`\x01`\x80\x1B\x90\x04`\xFF\x16`\x02\x81\x11\x15a8\x8AWa8\x8Aa@\x19V[\x14a8\xA8W`@QcZ\xA0\xD5\xF1`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`\x01\x80\x82\x01\x80T\x91\x84\x01\x80Ts\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x19\x81\x16`\x01`\x01`\xA0\x1B\x03\x90\x94\x16\x93\x84\x17\x82U\x82T`\x01`\x01`@\x1B\x03`\x01`\xA0\x1B\x91\x82\x90\x04\x16\x02`\x01`\x01`\xE0\x1B\x03\x19\x90\x91\x16\x90\x93\x17\x92\x90\x92\x17\x90\x91U`\x03\x82\x01T`\t\x84\x01U`\x07\x83\x01\x80T`\x01`@\x1Bc\xFF\xFF\xFF\xFF\x89\x16\x02o\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\x19\x82\x16\x81\x17\x83U\x92T`\xFF`\x01`\xE0\x1B\x90\x91\x04\x16\x92p\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x19\x16p\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\x19\x90\x91\x16\x17`\x01`\x80\x1B\x83`\x02\x81\x11\x15a9\x8CWa9\x8Ca@\x19V[\x02\x17\x90UP_a9\x9B\x84a\t:V[`\x07\x84\x01\x80Tg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x19\x16`\x01`\x01`@\x1B\x03\x83\x16\x17\x90U\x82T`@Qc'\x8FyC`\xE1\x1B\x81R\x91\x92P`\x01`\x01`\xA0\x1B\x03\x89\x81\x16\x92cO\x1E\xF2\x86\x92a9\xED\x92\x16\x90\x89\x90`\x04\x01aM]V[_`@Q\x80\x83\x03\x81_\x87\x80;\x15\x80\x15a:\x04W__\xFD[PZ\xF1\x15\x80\x15a:\x16W=__>=_\xFD[PP`@\x80Qc\xFF\xFF\xFF\xFF\x8A\x81\x16\x82R`\x01`\x01`@\x1B\x03\x86\x16` \x83\x01R\x88\x16\x93P\x7F\xF5\x85\xE0L\x05\xD3\x96\x90\x11p$w\x83\xD3\xE5\xF0\xEE\x9C\x1D\xF20r\x98[P\xAF\x08\x9F^H\xB1\x9D\x92P\x01`@Q\x80\x91\x03\x90\xA2PPPPPPPV[`oT`\xFF\x16a:\x91W`@QcS\x86i\x81`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`o\x80T`\xFF\x19\x16\x90U`@Q\x7F\x1E^4\xEE\xA35\x01\xAE\xCF.\xBE\xC9\xFE\x0E\x88J@\x80Bu\xEA\x7F\xE1\x0B+\xA0\x84\xC87C\x08\xB3\x90_\x90\xA1V[_\x82\x81R`4` \x90\x81R`@\x80\x83 `\x01`\x01`\xA0\x1B\x03\x85\x16\x84R\x90\x91R\x90 T`\xFF\x16a\x0CxW`@Qcv\x15\xBE\x1F`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`@\x80Q`\x01`\x01`\xA0\x1B\x03\x84\x16`$\x82\x01R`D\x80\x82\x01\x84\x90R\x82Q\x80\x83\x03\x90\x91\x01\x81R`d\x90\x91\x01\x90\x91R` \x81\x01\x80Q{\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x16c\xA9\x05\x9C\xBB`\xE0\x1B\x17\x90Ra\x0C@\x90\x84\x90a<UV[`oT`\xFF\x16\x15a;\x92W`@Qc\x0B\xC0\x11\xFF`\xE2\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[`o\x80T`\xFF\x19\x16`\x01\x17\x90U`@Q\x7F\"a\xEF\xE5\xAE\xF6\xFE\xDC\x1F\xD1U\x0B%\xFA\xCC\x91\x81tV#\x04\x9Cy\x01(p0\xB9\xAD\x1AT\x97\x90_\x90\xA1V[\x80\x82]PPV[_g\xFF\xFF\xFF\xFF\0\0\0\x01`\x01`\x01`@\x1B\x03\x83\x16\x10\x80\x15a<\x05WPg\xFF\xFF\xFF\xFF\0\0\0\x01`@\x83\x90\x1C`\x01`\x01`@\x1B\x03\x16\x10[\x80\x15a<%WPg\xFF\xFF\xFF\xFF\0\0\0\x01`\x80\x83\x90\x1C`\x01`\x01`@\x1B\x03\x16\x10[\x80\x15a<<WPg\xFF\xFF\xFF\xFF\0\0\0\x01`\xC0\x83\x90\x1C\x10[\x15a<IWP`\x01\x91\x90PV[P_\x91\x90PV[\x91\x90PV[_a<\xA9\x82`@Q\x80`@\x01`@R\x80` \x81R` \x01\x7FSafeERC20: low-level call failed\x81RP\x85`\x01`\x01`\xA0\x1B\x03\x16a=&\x90\x92\x91\x90c\xFF\xFF\xFF\xFF\x16V[\x80Q\x90\x91P\x15a\x0C@W\x80\x80` \x01\x90Q\x81\x01\x90a<\xC7\x91\x90aL\x8AV[a\x0C@W`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`*`$\x82\x01R\x7FSafeERC20: ERC20 operation did n`D\x82\x01Ri\x1B\xDD\x08\x1C\xDDX\xD8\xD9YY`\xB2\x1B`d\x82\x01R`\x84\x01a\x13\x7FV[``a=4\x84\x84_\x85a=<V[\x94\x93PPPPV[``\x82G\x10\x15a=\x9DW`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`&`$\x82\x01R\x7FAddress: insufficient balance fo`D\x82\x01Re\x1C\x88\x18\xD8[\x1B`\xD2\x1B`d\x82\x01R`\x84\x01a\x13\x7FV[__\x86`\x01`\x01`\xA0\x1B\x03\x16\x85\x87`@Qa=\xB8\x91\x90aL8V[_`@Q\x80\x83\x03\x81\x85\x87Z\xF1\x92PPP=\x80_\x81\x14a=\xF2W`@Q\x91P`\x1F\x19`?=\x01\x16\x82\x01`@R=\x82R=_` \x84\x01>a=\xF7V[``\x91P[P\x91P\x91Pa\x12\xE6\x87\x83\x83\x87``\x83\x15a>qW\x82Q_\x03a>jW`\x01`\x01`\xA0\x1B\x03\x85\x16;a>jW`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`\x1D`$\x82\x01R\x7FAddress: call to non-contract\0\0\0`D\x82\x01R`d\x01a\x13\x7FV[P\x81a=4V[a=4\x83\x83\x81Q\x15a>\x86W\x81Q\x80\x83` \x01\xFD[\x80`@QbF\x1B\xCD`\xE5\x1B\x81R`\x04\x01a\x13\x7F\x91\x90aC\x1AV[a\x08\xB0\x80aM\x7F\x839\x01\x90V[\x805c\xFF\xFF\xFF\xFF\x81\x16\x81\x14a<PW__\xFD[_` \x82\x84\x03\x12\x15a>\xD0W__\xFD[a>\xD9\x82a>\xADV[\x93\x92PPPV[\x805`\x01`\x01`@\x1B\x03\x81\x16\x81\x14a<PW__\xFD[`\x01`\x01`\xA0\x1B\x03\x81\x16\x81\x14a'\xBBW__\xFD[\x805a<P\x81a>\xF6V[________a\x03\xE0\x89\x8B\x03\x12\x15a?-W__\xFD[a?6\x89a>\xADV[\x97Pa?D` \x8A\x01a>\xE0V[\x96Pa?R`@\x8A\x01a>\xE0V[\x95Pa?```\x8A\x01a>\xE0V[\x94P`\x80\x89\x015\x93P`\xA0\x89\x015\x92P`\xC0\x89\x015a?~\x81a>\xF6V[\x91Pa\x03\xE0\x89\x01\x8A\x10\x15a?\x90W__\xFD[`\xE0\x89\x01\x90P\x92\x95\x98P\x92\x95\x98\x90\x93\x96PV[_` \x82\x84\x03\x12\x15a?\xB3W__\xFD[P5\x91\x90PV[__`@\x83\x85\x03\x12\x15a?\xCBW__\xFD[a?\xD4\x83a>\xADV[\x91Pa?\xE2` \x84\x01a>\xE0V[\x90P\x92P\x92\x90PV[__`@\x83\x85\x03\x12\x15a?\xFCW__\xFD[\x825\x91P` \x83\x015a@\x0E\x81a>\xF6V[\x80\x91PP\x92P\x92\x90PV[cNH{q`\xE0\x1B_R`!`\x04R`$_\xFD[`\x03\x81\x10a@IWcNH{q`\xE0\x1B_R`!`\x04R`$_\xFD[\x90RV[`\x01`\x01`\xA0\x1B\x03\x88\x81\x16\x82R\x87\x16` \x82\x01R`\x01`\x01`@\x1B\x03\x86\x16`@\x82\x01R`\xE0\x81\x01a@\x81``\x83\x01\x87a@-V[\x93\x15\x15`\x80\x82\x01R`\xA0\x81\x01\x92\x90\x92R`\xC0\x90\x91\x01R\x94\x93PPPPV[__\x83`\x1F\x84\x01\x12a@\xAFW__\xFD[P\x815`\x01`\x01`@\x1B\x03\x81\x11\x15a@\xC5W__\xFD[` \x83\x01\x91P\x83` \x82\x85\x01\x01\x11\x15a@\xDCW__\xFD[\x92P\x92\x90PV[________`\xC0\x89\x8B\x03\x12\x15a@\xFAW__\xFD[aA\x03\x89a>\xADV[\x97PaA\x11` \x8A\x01a>\xADV[\x96P`@\x89\x015\x95P``\x89\x015\x94P`\x80\x89\x015`\x01`\x01`@\x1B\x03\x81\x11\x15aA9W__\xFD[aAE\x8B\x82\x8C\x01a@\x9FV[\x90\x95P\x93PP`\xA0\x89\x015`\x01`\x01`@\x1B\x03\x81\x11\x15aAcW__\xFD[aAo\x8B\x82\x8C\x01a@\x9FV[\x99\x9C\x98\x9BP\x96\x99P\x94\x97\x93\x96\x92\x95\x94PPPV[\x81Q`\x01`\x01`\xA0\x1B\x03\x16\x81Ra\x01\x80\x81\x01` \x83\x01QaA\xAF` \x84\x01\x82`\x01`\x01`@\x1B\x03\x16\x90RV[P`@\x83\x01QaA\xCA`@\x84\x01\x82`\x01`\x01`\xA0\x1B\x03\x16\x90RV[P``\x83\x01QaA\xE5``\x84\x01\x82`\x01`\x01`@\x1B\x03\x16\x90RV[P`\x80\x83\x01Q`\x80\x83\x01R`\xA0\x83\x01QaB\n`\xA0\x84\x01\x82`\x01`\x01`@\x1B\x03\x16\x90RV[P`\xC0\x83\x01QaB%`\xC0\x84\x01\x82`\x01`\x01`@\x1B\x03\x16\x90RV[P`\xE0\x83\x01QaB@`\xE0\x84\x01\x82`\x01`\x01`@\x1B\x03\x16\x90RV[Pa\x01\0\x83\x01QaB]a\x01\0\x84\x01\x82`\x01`\x01`@\x1B\x03\x16\x90RV[Pa\x01 \x83\x01QaBra\x01 \x84\x01\x82a@-V[Pa\x01@\x83\x01Qa\x01@\x83\x01Ra\x01`\x83\x01Qa\x01`\x83\x01R\x92\x91PPV[______`\xC0\x87\x89\x03\x12\x15aB\xA6W__\xFD[aB\xAF\x87a>\xADV[\x95PaB\xBD` \x88\x01a>\xE0V[\x94PaB\xCB`@\x88\x01a>\xE0V[\x95\x98\x94\x97P\x94\x95``\x81\x015\x95P`\x80\x81\x015\x94`\xA0\x90\x91\x015\x93P\x91PPV[_\x81Q\x80\x84R\x80` \x84\x01` \x86\x01^_` \x82\x86\x01\x01R` `\x1F\x19`\x1F\x83\x01\x16\x85\x01\x01\x91PP\x92\x91PPV[` \x81R_a>\xD9` \x83\x01\x84aB\xECV[_` \x82\x84\x03\x12\x15aC<W__\xFD[a>\xD9\x82a>\xE0V[__`@\x83\x85\x03\x12\x15aCVW__\xFD[\x825a?\xD4\x81a>\xF6V[__`@\x83\x85\x03\x12\x15aCrW__\xFD[aC{\x83a>\xE0V[\x94` \x93\x90\x93\x015\x93PPPV[\x805`\x03\x81\x10a<PW__\xFD[cNH{q`\xE0\x1B_R`A`\x04R`$_\xFD[_\x82`\x1F\x83\x01\x12aC\xBAW__\xFD[\x815` \x83\x01__`\x01`\x01`@\x1B\x03\x84\x11\x15aC\xD9WaC\xD9aC\x97V[P`@Q`\x1F\x19`\x1F\x85\x01\x81\x16`?\x01\x16\x81\x01\x81\x81\x10`\x01`\x01`@\x1B\x03\x82\x11\x17\x15aD\x07WaD\x07aC\x97V[`@R\x83\x81R\x90P\x80\x82\x84\x01\x87\x10\x15aD\x1EW__\xFD[\x83\x83` \x83\x017_` \x85\x83\x01\x01R\x80\x94PPPPP\x92\x91PPV[_______`\xE0\x88\x8A\x03\x12\x15aDPW__\xFD[\x875aD[\x81a>\xF6V[\x96P` \x88\x015aDk\x81a>\xF6V[\x95PaDy`@\x89\x01a>\xE0V[\x94PaD\x87``\x89\x01aC\x89V[\x93P`\x80\x88\x015\x92P`\xA0\x88\x015`\x01`\x01`@\x1B\x03\x81\x11\x15aD\xA8W__\xFD[aD\xB4\x8A\x82\x8B\x01aC\xABV[\x97\x9A\x96\x99P\x94\x97\x93\x96\x92\x95\x92\x94PPP`\xC0\x90\x91\x015\x90V[___``\x84\x86\x03\x12\x15aD\xDFW__\xFD[\x835aD\xEA\x81a>\xF6V[\x92PaD\xF8` \x85\x01a>\xADV[\x91P`@\x84\x015`\x01`\x01`@\x1B\x03\x81\x11\x15aE\x12W__\xFD[aE\x1E\x86\x82\x87\x01aC\xABV[\x91PP\x92P\x92P\x92V[________a\x01\0\x89\x8B\x03\x12\x15aE@W__\xFD[aEI\x89a>\xADV[\x97PaEW` \x8A\x01a>\xE0V[\x96PaEe`@\x8A\x01a?\nV[\x95PaEs``\x8A\x01a?\nV[\x94PaE\x81`\x80\x8A\x01a?\nV[\x93P`\xA0\x89\x015`\x01`\x01`@\x1B\x03\x81\x11\x15aE\x9BW__\xFD[aE\xA7\x8B\x82\x8C\x01aC\xABV[\x93PP`\xC0\x89\x015`\x01`\x01`@\x1B\x03\x81\x11\x15aE\xC2W__\xFD[aE\xCE\x8B\x82\x8C\x01aC\xABV[\x92PP`\xE0\x89\x015`\x01`\x01`@\x1B\x03\x81\x11\x15aE\xE9W__\xFD[aE\xF5\x8B\x82\x8C\x01aC\xABV[\x91PP\x92\x95\x98P\x92\x95\x98\x90\x93\x96PV[_` \x82\x84\x03\x12\x15aF\x15W__\xFD[\x815a>\xD9\x81a>\xF6V[______`\xA0\x87\x89\x03\x12\x15aF5W__\xFD[aF>\x87a>\xADV[\x95P` \x87\x015\x94P`@\x87\x015\x93P``\x87\x015\x92P`\x80\x87\x015`\x01`\x01`@\x1B\x03\x81\x11\x15aFmW__\xFD[aFy\x89\x82\x8A\x01a@\x9FV[\x97\x9A\x96\x99P\x94\x97P\x92\x95\x93\x94\x92PPPV[__`@\x83\x85\x03\x12\x15aF\x9CW__\xFD[\x825aF\xA7\x81a>\xF6V[\x91Pa?\xE2` \x84\x01a>\xADV[`\x01`\x01`\xA0\x1B\x03\x8D\x81\x16\x82R`\x01`\x01`@\x1B\x03\x8D\x81\x16` \x84\x01R\x90\x8C\x16`@\x83\x01R\x8A\x81\x16``\x83\x01R`\x80\x82\x01\x8A\x90R\x88\x81\x16`\xA0\x83\x01R\x87\x16`\xC0\x82\x01Ra\x01\x80\x81\x01`\x01`\x01`@\x1B\x03\x87\x16`\xE0\x83\x01R`\x01`\x01`@\x1B\x03\x86\x16a\x01\0\x83\x01RaG*a\x01 \x83\x01\x86a@-V[a\x01@\x82\x01\x93\x90\x93Ra\x01`\x01R\x9A\x99PPPPPPPPPPV[________a\x01\0\x89\x8B\x03\x12\x15aG^W__\xFD[\x885aGi\x81a>\xF6V[\x97P` \x89\x015aGy\x81a>\xF6V[\x96PaG\x87`@\x8A\x01a>\xE0V[\x95PaG\x95``\x8A\x01a>\xE0V[\x94P`\x80\x89\x015\x93PaG\xAA`\xA0\x8A\x01aC\x89V[\x97\x9A\x96\x99P\x94\x97\x93\x96\x92\x95\x92\x94PPP`\xC0\x82\x015\x91`\xE0\x015\x90V[\x81Q`\x01`\x01`\xA0\x1B\x03\x16\x81Ra\x01\x80\x81\x01` \x83\x01QaG\xF3` \x84\x01\x82`\x01`\x01`@\x1B\x03\x16\x90RV[P`@\x83\x01QaH\x0E`@\x84\x01\x82`\x01`\x01`\xA0\x1B\x03\x16\x90RV[P``\x83\x01QaH)``\x84\x01\x82`\x01`\x01`@\x1B\x03\x16\x90RV[P`\x80\x83\x01Q`\x80\x83\x01R`\xA0\x83\x01QaHN`\xA0\x84\x01\x82`\x01`\x01`@\x1B\x03\x16\x90RV[P`\xC0\x83\x01QaHi`\xC0\x84\x01\x82`\x01`\x01`@\x1B\x03\x16\x90RV[P`\xE0\x83\x01QaH\x84`\xE0\x84\x01\x82`\x01`\x01`@\x1B\x03\x16\x90RV[Pa\x01\0\x83\x01QaH\xA1a\x01\0\x84\x01\x82`\x01`\x01`@\x1B\x03\x16\x90RV[Pa\x01 \x83\x01QaH\xBEa\x01 \x84\x01\x82`\x01`\x01`@\x1B\x03\x16\x90RV[Pa\x01@\x83\x01QaH\xDBa\x01@\x84\x01\x82`\x01`\x01`@\x1B\x03\x16\x90RV[Pa\x01`\x83\x01QaH\xF0a\x01`\x84\x01\x82a@-V[P\x92\x91PPV[cNH{q`\xE0\x1B_R`\x11`\x04R`$_\xFD[`\x01`\x01`@\x1B\x03\x81\x81\x16\x83\x82\x16\x01\x90\x81\x11\x15a\tcWa\tcaH\xF7V[\x80\x82\x02\x81\x15\x82\x82\x04\x84\x14\x17a\tcWa\tcaH\xF7V[_` \x82\x84\x03\x12\x15aIQW__\xFD[PQ\x91\x90PV[\x81\x83R\x81\x81` \x85\x017P_\x82\x82\x01` \x90\x81\x01\x91\x90\x91R`\x1F\x90\x91\x01`\x1F\x19\x16\x90\x91\x01\x01\x90V[`@\x81R_aI\x92`@\x83\x01\x86aB\xECV[\x82\x81\x03` \x84\x01RaI\xA5\x81\x85\x87aIXV[\x96\x95PPPPPPV[\x84\x81R``` \x82\x01R_aI\xC7``\x83\x01\x86aB\xECV[\x82\x81\x03`@\x84\x01Ra\x12\xE6\x81\x85\x87aIXV[` \x81R_a=4` \x83\x01\x84\x86aIXV[_` \x82\x84\x03\x12\x15aI\xFDW__\xFD[\x81Qa>\xD9\x81a>\xF6V[`\x01`\x01`@\x1B\x03\x82\x81\x16\x82\x82\x16\x03\x90\x81\x11\x15a\tcWa\tcaH\xF7V[cNH{q`\xE0\x1B_R`\x12`\x04R`$_\xFD[_\x82aJIWaJIaJ'V[P\x04\x90V[\x80\x82\x01\x80\x82\x11\x15a\tcWa\tcaH\xF7V[cNH{q`\xE0\x1B_R`2`\x04R`$_\xFD[_\x82aJ\x83WaJ\x83aJ'V[P\x06\x90V[\x81\x81\x03\x81\x81\x11\x15a\tcWa\tcaH\xF7V[_\x81aJ\xA9WaJ\xA9aH\xF7V[P_\x19\x01\x90V[_c\xFF\xFF\xFF\xFF\x82\x16c\xFF\xFF\xFF\xFF\x81\x03aJ\xCBWaJ\xCBaH\xF7V[`\x01\x01\x92\x91PPV[`\x01`\x01`\xA0\x1B\x03\x88\x16\x81R`\x01`\x01`\xA0\x1B\x03\x87\x16` \x82\x01R`\x01`\x01`@\x1B\x03\x86\x16`@\x82\x01RaK\x0B``\x82\x01\x86a@-V[\x83`\x80\x82\x01R`\xE0`\xA0\x82\x01R_aK&`\xE0\x83\x01\x85aB\xECV[\x90P\x82`\xC0\x83\x01R\x98\x97PPPPPPPPV[`\x01`\x01`\xA0\x1B\x03\x84\x16\x81R`\x01`\x01`\xA0\x1B\x03\x83\x16` \x82\x01R```@\x82\x01R_aKj``\x83\x01\x84aB\xECV[\x95\x94PPPPPV[`\x01`\x01`\xA0\x1B\x03\x87\x16\x81R`\x01`\x01`\xA0\x1B\x03\x86\x16` \x82\x01Rc\xFF\xFF\xFF\xFF\x85\x16`@\x82\x01R`\x01`\x01`\xA0\x1B\x03\x84\x16``\x82\x01R`\xC0`\x80\x82\x01R_aK\xBE`\xC0\x83\x01\x85aB\xECV[\x82\x81\x03`\xA0\x84\x01RaK\xD0\x81\x85aB\xECV[\x99\x98PPPPPPPPPV[`\x01`\x01`@\x1B\x03\x88\x81\x16\x82R`\x01`\x01`\xA0\x1B\x03\x88\x16` \x83\x01R\x86\x16`@\x82\x01R`\xE0\x81\x01aL\x11``\x83\x01\x87a@-V[`\x01`\x01`@\x1B\x03\x85\x16`\x80\x83\x01R\x83`\xA0\x83\x01R\x82`\xC0\x83\x01R\x98\x97PPPPPPPPV[_\x82Q\x80` \x85\x01\x84^_\x92\x01\x91\x82RP\x91\x90PV[a\x03 \x81\x01a\x03\0\x84\x837a\x03\0\x82\x01\x83_[`\x01\x81\x10\x15aL\x80W\x81Q\x83R` \x92\x83\x01\x92\x90\x91\x01\x90`\x01\x01aLaV[PPP\x93\x92PPPV[_` \x82\x84\x03\x12\x15aL\x9AW__\xFD[\x81Q\x80\x15\x15\x81\x14a>\xD9W__\xFD[k\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x19\x8B``\x1B\x16\x81R\x89`\x14\x82\x01R\x88`4\x82\x01R`\x01`\x01`@\x1B\x03`\xC0\x1B\x88`\xC0\x1B\x16`T\x82\x01R`\x01`\x01`@\x1B\x03`\xC0\x1B\x87`\xC0\x1B\x16`\\\x82\x01R`\x01`\x01`@\x1B\x03`\xC0\x1B\x86`\xC0\x1B\x16`d\x82\x01R\x84`l\x82\x01R\x83`\x8C\x82\x01R\x82`\xAC\x82\x01RaML`\xCC\x82\x01\x83`\xC0\x1B\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x16\x90RV[`\xD4\x01\x9A\x99PPPPPPPPPPV[`\x01`\x01`\xA0\x1B\x03\x83\x16\x81R`@` \x82\x01R_a=4`@\x83\x01\x84aB\xECV\xFE`\xA0`@R`@Qa\x08\xB08\x03\x80a\x08\xB0\x839\x81\x01`@\x81\x90Ra\0\"\x91a\x03'V[\x82\x81a\0.\x82\x82a\0VV[PP`\x01`\x01`\xA0\x1B\x03\x82\x16`\x80Ra\0Na\0I`\x80Q\x90V[a\0\xB4V[PPPa\x04\x0EV[a\0_\x82a\x01!V[`@Q`\x01`\x01`\xA0\x1B\x03\x83\x16\x90\x7F\xBC|\xD7Z \xEE'\xFD\x9A\xDE\xBA\xB3 A\xF7U!M\xBCk\xFF\xA9\x0C\xC0\"[9\xDA.\\-;\x90_\x90\xA2\x80Q\x15a\0\xA8Wa\0\xA3\x82\x82a\x01\x9FV[PPPV[a\0\xB0a\x02\x12V[PPV[\x7F~dMyB/\x17\xC0\x1EH\x94\xB5\xF4\xF5\x88\xD31\xEB\xFA(e=B\xAE\x83-\xC5\x9E8\xC9y\x8Fa\0\xF3_Q` a\x08\x90_9_Q\x90_RT`\x01`\x01`\xA0\x1B\x03\x16\x90V[`@\x80Q`\x01`\x01`\xA0\x1B\x03\x92\x83\x16\x81R\x91\x84\x16` \x83\x01R\x01`@Q\x80\x91\x03\x90\xA1a\x01\x1E\x81a\x023V[PV[\x80`\x01`\x01`\xA0\x1B\x03\x16;_\x03a\x01[W`@QcL\x9C\x8C\xE3`\xE0\x1B\x81R`\x01`\x01`\xA0\x1B\x03\x82\x16`\x04\x82\x01R`$\x01[`@Q\x80\x91\x03\x90\xFD[\x80\x7F6\x08\x94\xA1;\xA1\xA3!\x06g\xC8(I-\xB9\x8D\xCA> v\xCC75\xA9 \xA3\xCAP]8+\xBC[\x80T`\x01`\x01`\xA0\x1B\x03\x19\x16`\x01`\x01`\xA0\x1B\x03\x92\x90\x92\x16\x91\x90\x91\x17\x90UPV[``__\x84`\x01`\x01`\xA0\x1B\x03\x16\x84`@Qa\x01\xBB\x91\x90a\x03\xF8V[_`@Q\x80\x83\x03\x81\x85Z\xF4\x91PP=\x80_\x81\x14a\x01\xF3W`@Q\x91P`\x1F\x19`?=\x01\x16\x82\x01`@R=\x82R=_` \x84\x01>a\x01\xF8V[``\x91P[P\x90\x92P\x90Pa\x02\t\x85\x83\x83a\x02pV[\x95\x94PPPPPV[4\x15a\x021W`@Qc\xB3\x98\x97\x9F`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[V[`\x01`\x01`\xA0\x1B\x03\x81\x16a\x02\\W`@Qc1s\xBD\xD1`\xE1\x1B\x81R_`\x04\x82\x01R`$\x01a\x01RV[\x80_Q` a\x08\x90_9_Q\x90_Ra\x01~V[``\x82a\x02\x85Wa\x02\x80\x82a\x02\xCFV[a\x02\xC8V[\x81Q\x15\x80\x15a\x02\x9CWP`\x01`\x01`\xA0\x1B\x03\x84\x16;\x15[\x15a\x02\xC5W`@Qc\x99\x96\xB3\x15`\xE0\x1B\x81R`\x01`\x01`\xA0\x1B\x03\x85\x16`\x04\x82\x01R`$\x01a\x01RV[P\x80[\x93\x92PPPV[\x80Q\x15a\x02\xDFW\x80Q\x80\x82` \x01\xFD[`@Qc\n\x12\xF5!`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x80Q`\x01`\x01`\xA0\x1B\x03\x81\x16\x81\x14a\x03\x0EW__\xFD[\x91\x90PV[cNH{q`\xE0\x1B_R`A`\x04R`$_\xFD[___``\x84\x86\x03\x12\x15a\x039W__\xFD[a\x03B\x84a\x02\xF8V[\x92Pa\x03P` \x85\x01a\x02\xF8V[`@\x85\x01Q\x90\x92P`\x01`\x01`@\x1B\x03\x81\x11\x15a\x03kW__\xFD[\x84\x01`\x1F\x81\x01\x86\x13a\x03{W__\xFD[\x80Q`\x01`\x01`@\x1B\x03\x81\x11\x15a\x03\x94Wa\x03\x94a\x03\x13V[`@Q`\x1F\x82\x01`\x1F\x19\x90\x81\x16`?\x01\x16\x81\x01`\x01`\x01`@\x1B\x03\x81\x11\x82\x82\x10\x17\x15a\x03\xC2Wa\x03\xC2a\x03\x13V[`@R\x81\x81R\x82\x82\x01` \x01\x88\x10\x15a\x03\xD9W__\xFD[\x81` \x84\x01` \x83\x01^_` \x83\x83\x01\x01R\x80\x93PPPP\x92P\x92P\x92V[_\x82Q\x80` \x85\x01\x84^_\x92\x01\x91\x82RP\x91\x90PV[`\x80Qa\x04ka\x04%_9_`\x10\x01Ra\x04k_\xF3\xFE`\x80`@Ra\0\x0Ca\0\x0EV[\0[\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`\x01`\x01`\xA0\x1B\x03\x163\x03a\0\x81W_5\x7F\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x16c'\x8FyC`\xE1\x1B\x14a\0yWa\0wa\0\x85V[V[a\0wa\0\x95V[a\0w[a\0wa\0\x90a\0\xC3V[a\0\xFAV[_\x80a\0\xA46`\x04\x81\x84a\x03\x13V[\x81\x01\x90a\0\xB1\x91\x90a\x03NV[\x91P\x91Pa\0\xBF\x82\x82a\x01\x18V[PPV[_a\0\xF5\x7F6\x08\x94\xA1;\xA1\xA3!\x06g\xC8(I-\xB9\x8D\xCA> v\xCC75\xA9 \xA3\xCAP]8+\xBCT`\x01`\x01`\xA0\x1B\x03\x16\x90V[\x90P\x90V[6__7__6_\x84Z\xF4=__>\x80\x80\x15a\x01\x14W=_\xF3[=_\xFD[a\x01!\x82a\x01rV[`@Q`\x01`\x01`\xA0\x1B\x03\x83\x16\x90\x7F\xBC|\xD7Z \xEE'\xFD\x9A\xDE\xBA\xB3 A\xF7U!M\xBCk\xFF\xA9\x0C\xC0\"[9\xDA.\\-;\x90_\x90\xA2\x80Q\x15a\x01jWa\x01e\x82\x82a\x01\xFAV[PPPV[a\0\xBFa\x02lV[\x80`\x01`\x01`\xA0\x1B\x03\x16;_\x03a\x01\xACW`@QcL\x9C\x8C\xE3`\xE0\x1B\x81R`\x01`\x01`\xA0\x1B\x03\x82\x16`\x04\x82\x01R`$\x01[`@Q\x80\x91\x03\x90\xFD[\x7F6\x08\x94\xA1;\xA1\xA3!\x06g\xC8(I-\xB9\x8D\xCA> v\xCC75\xA9 \xA3\xCAP]8+\xBC\x80Ts\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x19\x16`\x01`\x01`\xA0\x1B\x03\x92\x90\x92\x16\x91\x90\x91\x17\x90UV[``__\x84`\x01`\x01`\xA0\x1B\x03\x16\x84`@Qa\x02\x16\x91\x90a\x04\x1FV[_`@Q\x80\x83\x03\x81\x85Z\xF4\x91PP=\x80_\x81\x14a\x02NW`@Q\x91P`\x1F\x19`?=\x01\x16\x82\x01`@R=\x82R=_` \x84\x01>a\x02SV[``\x91P[P\x91P\x91Pa\x02c\x85\x83\x83a\x02\x8BV[\x95\x94PPPPPV[4\x15a\0wW`@Qc\xB3\x98\x97\x9F`\xE0\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[``\x82a\x02\xA0Wa\x02\x9B\x82a\x02\xEAV[a\x02\xE3V[\x81Q\x15\x80\x15a\x02\xB7WP`\x01`\x01`\xA0\x1B\x03\x84\x16;\x15[\x15a\x02\xE0W`@Qc\x99\x96\xB3\x15`\xE0\x1B\x81R`\x01`\x01`\xA0\x1B\x03\x85\x16`\x04\x82\x01R`$\x01a\x01\xA3V[P\x80[\x93\x92PPPV[\x80Q\x15a\x02\xFAW\x80Q\x80\x82` \x01\xFD[`@Qc\n\x12\xF5!`\xE1\x1B\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[__\x85\x85\x11\x15a\x03!W__\xFD[\x83\x86\x11\x15a\x03-W__\xFD[PP\x82\x01\x93\x91\x90\x92\x03\x91PV[cNH{q`\xE0\x1B_R`A`\x04R`$_\xFD[__`@\x83\x85\x03\x12\x15a\x03_W__\xFD[\x825`\x01`\x01`\xA0\x1B\x03\x81\x16\x81\x14a\x03uW__\xFD[\x91P` \x83\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x03\x90W__\xFD[\x83\x01`\x1F\x81\x01\x85\x13a\x03\xA0W__\xFD[\x805g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x03\xBAWa\x03\xBAa\x03:V[`@Q`\x1F\x82\x01`\x1F\x19\x90\x81\x16`?\x01\x16\x81\x01g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x82\x82\x10\x17\x15a\x03\xE9Wa\x03\xE9a\x03:V[`@R\x81\x81R\x82\x82\x01` \x01\x87\x10\x15a\x04\0W__\xFD[\x81` \x84\x01` \x83\x017_` \x83\x83\x01\x01R\x80\x93PPPP\x92P\x92\x90PV[_\x82Q\x80` \x85\x01\x84^_\x92\x01\x91\x82RP\x91\x90PV\xFE\xA2dipfsX\"\x12 \xF0\xC4\x0B\x8D\x0B\xC5ZE\x01:q\xEE?\xD7&L\xE8s\xFC\xC5a\xD0JA\xBA\x1F$\xAD\xBC\xEF\x95\x8CdsolcC\0\x08\x1C\x003\xB51'hJV\x8B1s\xAE\x13\xB9\xF8\xA6\x01n$>c\xB6\xE8\xEE\x11x\xD6\xA7\x17\x85\x0B]a\x03\xA2dipfsX\"\x12 :<\x88\xF2\x08\xED\xABu\xCBuw\xE6j\xB9\x8D\xC6\x12\xD1\xC8\x03\x8C\xB4\xBD\xFD\xA5\xCC\\\xF9G\xA1\xD8\x91dsolcC\0\x08\x1C\x003";
    /// The deployed bytecode of the contract.
    pub static POLYGONROLLUPMANAGER_DEPLOYED_BYTECODE: ::ethers::core::types::Bytes = ::ethers::core::types::Bytes::from_static(
        __DEPLOYED_BYTECODE,
    );
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
        /// Constructs the general purpose `Deployer` instance based on the provided constructor arguments and sends it.
        /// Returns a new instance of a deployer that returns an instance of this contract after sending the transaction
        ///
        /// Notes:
        /// - If there are no constructor arguments, you should pass `()` as the argument.
        /// - The default poll duration is 7 seconds.
        /// - The default number of confirmations is 1 block.
        ///
        ///
        /// # Example
        ///
        /// Generate contract bindings with `abigen!` and deploy a new contract instance.
        ///
        /// *Note*: this requires a `bytecode` and `abi` object in the `greeter.json` artifact.
        ///
        /// ```ignore
        /// # async fn deploy<M: ethers::providers::Middleware>(client: ::std::sync::Arc<M>) {
        ///     abigen!(Greeter, "../greeter.json");
        ///
        ///    let greeter_contract = Greeter::deploy(client, "Hello world!".to_string()).unwrap().send().await.unwrap();
        ///    let msg = greeter_contract.greet().call().await.unwrap();
        /// # }
        /// ```
        pub fn deploy<T: ::ethers::core::abi::Tokenize>(
            client: ::std::sync::Arc<M>,
            constructor_args: T,
        ) -> ::core::result::Result<
            ::ethers::contract::builders::ContractDeployer<M, Self>,
            ::ethers::contract::ContractError<M>,
        > {
            let factory = ::ethers::contract::ContractFactory::new(
                POLYGONROLLUPMANAGER_ABI.clone(),
                POLYGONROLLUPMANAGER_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ::ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
        }
        ///Calls the contract's `DEFAULT_ADMIN_ROLE` (0xa217fddf) function
        pub fn default_admin_role(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([162, 23, 253, 223], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `ROLLUP_MANAGER_VERSION` (0xd8905812) function
        pub fn rollup_manager_version(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::std::string::String> {
            self.0
                .method_hash([216, 144, 88, 18], ())
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
        ///Calls the contract's `addExistingRollup` (0xe80e5030) function
        pub fn add_existing_rollup(
            &self,
            rollup_address: ::ethers::core::types::Address,
            verifier: ::ethers::core::types::Address,
            fork_id: u64,
            chain_id: u64,
            init_root: [u8; 32],
            rollup_verifier_type: u8,
            program_v_key: [u8; 32],
            init_pessimistic_root: [u8; 32],
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [232, 14, 80, 48],
                    (
                        rollup_address,
                        verifier,
                        fork_id,
                        chain_id,
                        init_root,
                        rollup_verifier_type,
                        program_v_key,
                        init_pessimistic_root,
                    ),
                )
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `addNewRollupType` (0xabcb5198) function
        pub fn add_new_rollup_type(
            &self,
            consensus_implementation: ::ethers::core::types::Address,
            verifier: ::ethers::core::types::Address,
            fork_id: u64,
            rollup_verifier_type: u8,
            genesis: [u8; 32],
            description: ::std::string::String,
            program_v_key: [u8; 32],
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [171, 203, 81, 152],
                    (
                        consensus_implementation,
                        verifier,
                        fork_id,
                        rollup_verifier_type,
                        genesis,
                        description,
                        program_v_key,
                    ),
                )
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `aggLayerGateway` (0xab0475cf) function
        pub fn agg_layer_gateway(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Address,
        > {
            self.0
                .method_hash([171, 4, 117, 207], ())
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
        ///Calls the contract's `createNewRollup` (0xc5b4fdb6) function
        pub fn create_new_rollup(
            &self,
            rollup_type_id: u32,
            chain_id: u64,
            admin: ::ethers::core::types::Address,
            sequencer: ::ethers::core::types::Address,
            gas_token_address: ::ethers::core::types::Address,
            sequencer_url: ::std::string::String,
            network_name: ::std::string::String,
            initialize_bytes_custom_chain: ::ethers::core::types::Bytes,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [197, 180, 253, 182],
                    (
                        rollup_type_id,
                        chain_id,
                        admin,
                        sequencer,
                        gas_token_address,
                        sequencer_url,
                        network_name,
                        initialize_bytes_custom_chain,
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
        ///Calls the contract's `getInputPessimisticBytes` (0xdd0464b9) function
        pub fn get_input_pessimistic_bytes(
            &self,
            rollup_id: u32,
            l_1_info_tree_root: [u8; 32],
            new_local_exit_root: [u8; 32],
            new_pessimistic_root: [u8; 32],
            custom_chain_data: ::ethers::core::types::Bytes,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Bytes,
        > {
            self.0
                .method_hash(
                    [221, 4, 100, 185],
                    (
                        rollup_id,
                        l_1_info_tree_root,
                        new_local_exit_root,
                        new_pessimistic_root,
                        custom_chain_data,
                    ),
                )
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
        ///Calls the contract's `initialize` (0x8129fc1c) function
        pub fn initialize(&self) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([129, 41, 252, 28], ())
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
        ///Calls the contract's `rollbackBatches` (0x8fd88cc2) function
        pub fn rollback_batches(
            &self,
            rollup_contract: ::ethers::core::types::Address,
            target_batch: u64,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([143, 216, 140, 194], (rollup_contract, target_batch))
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
        ) -> ::ethers::contract::builders::ContractCall<M, RollupDataReturn> {
            self.0
                .method_hash([249, 196, 194, 174], rollup_id)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `rollupIDToRollupDataDeserialized` (0xe4f3d8f9) function
        pub fn rollup_id_to_rollup_data_deserialized(
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
                u8,
                [u8; 32],
                [u8; 32],
            ),
        > {
            self.0
                .method_hash([228, 243, 216, 249], rollup_id)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `rollupIDToRollupDataV2` (0x74d9c244) function
        pub fn rollup_id_to_rollup_data_v2(
            &self,
            rollup_id: u32,
        ) -> ::ethers::contract::builders::ContractCall<M, RollupDataReturnV2> {
            self.0
                .method_hash([116, 217, 194, 68], rollup_id)
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
        ///Calls the contract's `updateRollupByRollupAdmin` (0xdfdb8c5e) function
        pub fn update_rollup_by_rollup_admin(
            &self,
            rollup_contract: ::ethers::core::types::Address,
            new_rollup_type_id: u32,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([223, 219, 140, 94], (rollup_contract, new_rollup_type_id))
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
        ///Calls the contract's `verifyPessimisticTrustedAggregator` (0x6c766877) function
        pub fn verify_pessimistic_trusted_aggregator(
            &self,
            rollup_id: u32,
            l_1_info_tree_leaf_count: u32,
            new_local_exit_root: [u8; 32],
            new_pessimistic_root: [u8; 32],
            proof: ::ethers::core::types::Bytes,
            custom_chain_data: ::ethers::core::types::Bytes,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [108, 118, 104, 119],
                    (
                        rollup_id,
                        l_1_info_tree_leaf_count,
                        new_local_exit_root,
                        new_pessimistic_root,
                        proof,
                        custom_chain_data,
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
        ///Gets the contract's `RollbackBatches` event
        pub fn rollback_batches_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            RollbackBatchesFilter,
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
        ///Gets the contract's `UpdateRollupManagerVersion` event
        pub fn update_rollup_manager_version_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            UpdateRollupManagerVersionFilter,
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
    ///Custom Error type `AllBatchesMustBeVerified` with signature `AllBatchesMustBeVerified()` and selector `0x44541072`
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
    #[etherror(name = "AllBatchesMustBeVerified", abi = "AllBatchesMustBeVerified()")]
    pub struct AllBatchesMustBeVerified;
    ///Custom Error type `AllSequencedMustBeVerified` with signature `AllSequencedMustBeVerified()` and selector `0xcc862d4a`
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
        name = "AllSequencedMustBeVerified",
        abi = "AllSequencedMustBeVerified()"
    )]
    pub struct AllSequencedMustBeVerified;
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
    ///Custom Error type `CannotUpdateWithUnconsolidatedPendingState` with signature `CannotUpdateWithUnconsolidatedPendingState()` and selector `0x9d59507b`
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
        name = "CannotUpdateWithUnconsolidatedPendingState",
        abi = "CannotUpdateWithUnconsolidatedPendingState()"
    )]
    pub struct CannotUpdateWithUnconsolidatedPendingState;
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
    ///Custom Error type `ChainIDOutOfRange` with signature `ChainIDOutOfRange()` and selector `0x4c753f57`
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
    #[etherror(name = "ChainIDOutOfRange", abi = "ChainIDOutOfRange()")]
    pub struct ChainIDOutOfRange;
    ///Custom Error type `EmptyVerifySequencesData` with signature `EmptyVerifySequencesData()` and selector `0x8a51facb`
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
    #[etherror(name = "EmptyVerifySequencesData", abi = "EmptyVerifySequencesData()")]
    pub struct EmptyVerifySequencesData;
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
    ///Custom Error type `FinalNumSequenceBelowLastVerifiedSequence` with signature `FinalNumSequenceBelowLastVerifiedSequence()` and selector `0x42f31f92`
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
        name = "FinalNumSequenceBelowLastVerifiedSequence",
        abi = "FinalNumSequenceBelowLastVerifiedSequence()"
    )]
    pub struct FinalNumSequenceBelowLastVerifiedSequence;
    ///Custom Error type `FinalNumSequenceDoesNotMatchPendingState` with signature `FinalNumSequenceDoesNotMatchPendingState()` and selector `0xb7d5b4a3`
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
        name = "FinalNumSequenceDoesNotMatchPendingState",
        abi = "FinalNumSequenceDoesNotMatchPendingState()"
    )]
    pub struct FinalNumSequenceDoesNotMatchPendingState;
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
    ///Custom Error type `InitSequenceMustMatchCurrentForkID` with signature `InitSequenceMustMatchCurrentForkID()` and selector `0xf5f2eb13`
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
        name = "InitSequenceMustMatchCurrentForkID",
        abi = "InitSequenceMustMatchCurrentForkID()"
    )]
    pub struct InitSequenceMustMatchCurrentForkID;
    ///Custom Error type `InitSequenceNumDoesNotMatchPendingState` with signature `InitSequenceNumDoesNotMatchPendingState()` and selector `0x686446b1`
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
        name = "InitSequenceNumDoesNotMatchPendingState",
        abi = "InitSequenceNumDoesNotMatchPendingState()"
    )]
    pub struct InitSequenceNumDoesNotMatchPendingState;
    ///Custom Error type `InvalidPessimisticProof` with signature `InvalidPessimisticProof()` and selector `0x52ad525a`
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
    #[etherror(name = "InvalidPessimisticProof", abi = "InvalidPessimisticProof()")]
    pub struct InvalidPessimisticProof;
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
    ///Custom Error type `InvalidRangeMultiplierZkGasPrice` with signature `InvalidRangeMultiplierZkGasPrice()` and selector `0x44ceee73`
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
        name = "InvalidRangeMultiplierZkGasPrice",
        abi = "InvalidRangeMultiplierZkGasPrice()"
    )]
    pub struct InvalidRangeMultiplierZkGasPrice;
    ///Custom Error type `InvalidRangeSequenceTimeTarget` with signature `InvalidRangeSequenceTimeTarget()` and selector `0xe04b5d74`
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
        name = "InvalidRangeSequenceTimeTarget",
        abi = "InvalidRangeSequenceTimeTarget()"
    )]
    pub struct InvalidRangeSequenceTimeTarget;
    ///Custom Error type `InvalidRollup` with signature `InvalidRollup()` and selector `0x43ba19f2`
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
    #[etherror(name = "InvalidRollup", abi = "InvalidRollup()")]
    pub struct InvalidRollup;
    ///Custom Error type `InvalidRollupType` with signature `InvalidRollupType()` and selector `0x63d722e7`
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
    #[etherror(name = "InvalidRollupType", abi = "InvalidRollupType()")]
    pub struct InvalidRollupType;
    ///Custom Error type `InvalidVerifierType` with signature `InvalidVerifierType()` and selector `0xe4ffd914`
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
    #[etherror(name = "InvalidVerifierType", abi = "InvalidVerifierType()")]
    pub struct InvalidVerifierType;
    ///Custom Error type `L1InfoTreeLeafCountInvalid` with signature `L1InfoTreeLeafCountInvalid()` and selector `0xa60721e1`
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
        name = "L1InfoTreeLeafCountInvalid",
        abi = "L1InfoTreeLeafCountInvalid()"
    )]
    pub struct L1InfoTreeLeafCountInvalid;
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
    ///Custom Error type `MustSequenceSomeBlob` with signature `MustSequenceSomeBlob()` and selector `0x562a9374`
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
    #[etherror(name = "MustSequenceSomeBlob", abi = "MustSequenceSomeBlob()")]
    pub struct MustSequenceSomeBlob;
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
    ///Custom Error type `NotAllowedAddress` with signature `NotAllowedAddress()` and selector `0x1a06d0fe`
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
    #[etherror(name = "NotAllowedAddress", abi = "NotAllowedAddress()")]
    pub struct NotAllowedAddress;
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
    ///Custom Error type `OnlyRollupAdmin` with signature `OnlyRollupAdmin()` and selector `0x696072e9`
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
    #[etherror(name = "OnlyRollupAdmin", abi = "OnlyRollupAdmin()")]
    pub struct OnlyRollupAdmin;
    ///Custom Error type `OnlyStateTransitionChains` with signature `OnlyStateTransitionChains()` and selector `0x90db0d07`
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
    #[etherror(name = "OnlyStateTransitionChains", abi = "OnlyStateTransitionChains()")]
    pub struct OnlyStateTransitionChains;
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
    ///Custom Error type `PendingStateNumExist` with signature `PendingStateNumExist()` and selector `0x60dbf8ae`
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
    #[etherror(name = "PendingStateNumExist", abi = "PendingStateNumExist()")]
    pub struct PendingStateNumExist;
    ///Custom Error type `ReentrancyGuardReentrantCall` with signature `ReentrancyGuardReentrantCall()` and selector `0x3ee5aeb5`
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
        name = "ReentrancyGuardReentrantCall",
        abi = "ReentrancyGuardReentrantCall()"
    )]
    pub struct ReentrancyGuardReentrantCall;
    ///Custom Error type `RollbackBatchIsNotEndOfSequence` with signature `RollbackBatchIsNotEndOfSequence()` and selector `0x9753965f`
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
        name = "RollbackBatchIsNotEndOfSequence",
        abi = "RollbackBatchIsNotEndOfSequence()"
    )]
    pub struct RollbackBatchIsNotEndOfSequence;
    ///Custom Error type `RollbackBatchIsNotValid` with signature `RollbackBatchIsNotValid()` and selector `0xcb23ebdf`
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
    #[etherror(name = "RollbackBatchIsNotValid", abi = "RollbackBatchIsNotValid()")]
    pub struct RollbackBatchIsNotValid;
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
    ///Custom Error type `RollupIDNotAscendingOrder` with signature `RollupIDNotAscendingOrder()` and selector `0x51fcf62a`
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
    #[etherror(name = "RollupIDNotAscendingOrder", abi = "RollupIDNotAscendingOrder()")]
    pub struct RollupIDNotAscendingOrder;
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
    ///Custom Error type `StateTransitionChainsNotAllowed` with signature `StateTransitionChainsNotAllowed()` and selector `0x5b6602b7`
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
        name = "StateTransitionChainsNotAllowed",
        abi = "StateTransitionChainsNotAllowed()"
    )]
    pub struct StateTransitionChainsNotAllowed;
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
    ///Custom Error type `UpdateToOldRollupTypeID` with signature `UpdateToOldRollupTypeID()` and selector `0x3e37e233`
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
    #[etherror(name = "UpdateToOldRollupTypeID", abi = "UpdateToOldRollupTypeID()")]
    pub struct UpdateToOldRollupTypeID;
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
    ///Custom Error type `zkGasPriceOfRange` with signature `zkGasPriceOfRange()` and selector `0x0c0bbd27`
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
    #[etherror(name = "zkGasPriceOfRange", abi = "zkGasPriceOfRange()")]
    pub struct zkGasPriceOfRange;
    ///Container type for all of the contract's custom errors
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum PolygonRollupManagerErrors {
        AccessControlOnlyCanRenounceRolesForSelf(
            AccessControlOnlyCanRenounceRolesForSelf,
        ),
        AddressDoNotHaveRequiredRole(AddressDoNotHaveRequiredRole),
        AllBatchesMustBeVerified(AllBatchesMustBeVerified),
        AllSequencedMustBeVerified(AllSequencedMustBeVerified),
        AllzkEVMSequencedBatchesMustBeVerified(AllzkEVMSequencedBatchesMustBeVerified),
        BatchFeeOutOfRange(BatchFeeOutOfRange),
        CannotUpdateWithUnconsolidatedPendingState(
            CannotUpdateWithUnconsolidatedPendingState,
        ),
        ChainIDAlreadyExist(ChainIDAlreadyExist),
        ChainIDOutOfRange(ChainIDOutOfRange),
        EmptyVerifySequencesData(EmptyVerifySequencesData),
        ExceedMaxVerifyBatches(ExceedMaxVerifyBatches),
        FinalNumBatchBelowLastVerifiedBatch(FinalNumBatchBelowLastVerifiedBatch),
        FinalNumBatchDoesNotMatchPendingState(FinalNumBatchDoesNotMatchPendingState),
        FinalNumSequenceBelowLastVerifiedSequence(
            FinalNumSequenceBelowLastVerifiedSequence,
        ),
        FinalNumSequenceDoesNotMatchPendingState(
            FinalNumSequenceDoesNotMatchPendingState,
        ),
        FinalPendingStateNumInvalid(FinalPendingStateNumInvalid),
        HaltTimeoutNotExpired(HaltTimeoutNotExpired),
        InitBatchMustMatchCurrentForkID(InitBatchMustMatchCurrentForkID),
        InitNumBatchAboveLastVerifiedBatch(InitNumBatchAboveLastVerifiedBatch),
        InitNumBatchDoesNotMatchPendingState(InitNumBatchDoesNotMatchPendingState),
        InitSequenceMustMatchCurrentForkID(InitSequenceMustMatchCurrentForkID),
        InitSequenceNumDoesNotMatchPendingState(InitSequenceNumDoesNotMatchPendingState),
        InvalidPessimisticProof(InvalidPessimisticProof),
        InvalidProof(InvalidProof),
        InvalidRangeBatchTimeTarget(InvalidRangeBatchTimeTarget),
        InvalidRangeMultiplierBatchFee(InvalidRangeMultiplierBatchFee),
        InvalidRangeMultiplierZkGasPrice(InvalidRangeMultiplierZkGasPrice),
        InvalidRangeSequenceTimeTarget(InvalidRangeSequenceTimeTarget),
        InvalidRollup(InvalidRollup),
        InvalidRollupType(InvalidRollupType),
        InvalidVerifierType(InvalidVerifierType),
        L1InfoTreeLeafCountInvalid(L1InfoTreeLeafCountInvalid),
        MustSequenceSomeBatch(MustSequenceSomeBatch),
        MustSequenceSomeBlob(MustSequenceSomeBlob),
        NewAccInputHashDoesNotExist(NewAccInputHashDoesNotExist),
        NewPendingStateTimeoutMustBeLower(NewPendingStateTimeoutMustBeLower),
        NewStateRootNotInsidePrime(NewStateRootNotInsidePrime),
        NewTrustedAggregatorTimeoutMustBeLower(NewTrustedAggregatorTimeoutMustBeLower),
        NotAllowedAddress(NotAllowedAddress),
        OldAccInputHashDoesNotExist(OldAccInputHashDoesNotExist),
        OldStateRootDoesNotExist(OldStateRootDoesNotExist),
        OnlyEmergencyState(OnlyEmergencyState),
        OnlyNotEmergencyState(OnlyNotEmergencyState),
        OnlyRollupAdmin(OnlyRollupAdmin),
        OnlyStateTransitionChains(OnlyStateTransitionChains),
        PendingStateDoesNotExist(PendingStateDoesNotExist),
        PendingStateInvalid(PendingStateInvalid),
        PendingStateNotConsolidable(PendingStateNotConsolidable),
        PendingStateNumExist(PendingStateNumExist),
        ReentrancyGuardReentrantCall(ReentrancyGuardReentrantCall),
        RollbackBatchIsNotEndOfSequence(RollbackBatchIsNotEndOfSequence),
        RollbackBatchIsNotValid(RollbackBatchIsNotValid),
        RollupAddressAlreadyExist(RollupAddressAlreadyExist),
        RollupIDNotAscendingOrder(RollupIDNotAscendingOrder),
        RollupMustExist(RollupMustExist),
        RollupTypeDoesNotExist(RollupTypeDoesNotExist),
        RollupTypeObsolete(RollupTypeObsolete),
        SenderMustBeRollup(SenderMustBeRollup),
        StateTransitionChainsNotAllowed(StateTransitionChainsNotAllowed),
        StoredRootMustBeDifferentThanNewRoot(StoredRootMustBeDifferentThanNewRoot),
        TrustedAggregatorTimeoutNotExpired(TrustedAggregatorTimeoutNotExpired),
        UpdateNotCompatible(UpdateNotCompatible),
        UpdateToOldRollupTypeID(UpdateToOldRollupTypeID),
        UpdateToSameRollupTypeID(UpdateToSameRollupTypeID),
        zkGasPriceOfRange(zkGasPriceOfRange),
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
            if let Ok(decoded) = <AllBatchesMustBeVerified as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::AllBatchesMustBeVerified(decoded));
            }
            if let Ok(decoded) = <AllSequencedMustBeVerified as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::AllSequencedMustBeVerified(decoded));
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
            if let Ok(decoded) = <CannotUpdateWithUnconsolidatedPendingState as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::CannotUpdateWithUnconsolidatedPendingState(decoded));
            }
            if let Ok(decoded) = <ChainIDAlreadyExist as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::ChainIDAlreadyExist(decoded));
            }
            if let Ok(decoded) = <ChainIDOutOfRange as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::ChainIDOutOfRange(decoded));
            }
            if let Ok(decoded) = <EmptyVerifySequencesData as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::EmptyVerifySequencesData(decoded));
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
            if let Ok(decoded) = <FinalNumSequenceBelowLastVerifiedSequence as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::FinalNumSequenceBelowLastVerifiedSequence(decoded));
            }
            if let Ok(decoded) = <FinalNumSequenceDoesNotMatchPendingState as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::FinalNumSequenceDoesNotMatchPendingState(decoded));
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
            if let Ok(decoded) = <InitSequenceMustMatchCurrentForkID as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::InitSequenceMustMatchCurrentForkID(decoded));
            }
            if let Ok(decoded) = <InitSequenceNumDoesNotMatchPendingState as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::InitSequenceNumDoesNotMatchPendingState(decoded));
            }
            if let Ok(decoded) = <InvalidPessimisticProof as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::InvalidPessimisticProof(decoded));
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
            if let Ok(decoded) = <InvalidRangeMultiplierZkGasPrice as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::InvalidRangeMultiplierZkGasPrice(decoded));
            }
            if let Ok(decoded) = <InvalidRangeSequenceTimeTarget as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::InvalidRangeSequenceTimeTarget(decoded));
            }
            if let Ok(decoded) = <InvalidRollup as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::InvalidRollup(decoded));
            }
            if let Ok(decoded) = <InvalidRollupType as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::InvalidRollupType(decoded));
            }
            if let Ok(decoded) = <InvalidVerifierType as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::InvalidVerifierType(decoded));
            }
            if let Ok(decoded) = <L1InfoTreeLeafCountInvalid as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::L1InfoTreeLeafCountInvalid(decoded));
            }
            if let Ok(decoded) = <MustSequenceSomeBatch as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::MustSequenceSomeBatch(decoded));
            }
            if let Ok(decoded) = <MustSequenceSomeBlob as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::MustSequenceSomeBlob(decoded));
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
            if let Ok(decoded) = <NotAllowedAddress as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::NotAllowedAddress(decoded));
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
            if let Ok(decoded) = <OnlyRollupAdmin as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::OnlyRollupAdmin(decoded));
            }
            if let Ok(decoded) = <OnlyStateTransitionChains as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::OnlyStateTransitionChains(decoded));
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
            if let Ok(decoded) = <PendingStateNumExist as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::PendingStateNumExist(decoded));
            }
            if let Ok(decoded) = <ReentrancyGuardReentrantCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::ReentrancyGuardReentrantCall(decoded));
            }
            if let Ok(decoded) = <RollbackBatchIsNotEndOfSequence as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::RollbackBatchIsNotEndOfSequence(decoded));
            }
            if let Ok(decoded) = <RollbackBatchIsNotValid as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::RollbackBatchIsNotValid(decoded));
            }
            if let Ok(decoded) = <RollupAddressAlreadyExist as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::RollupAddressAlreadyExist(decoded));
            }
            if let Ok(decoded) = <RollupIDNotAscendingOrder as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::RollupIDNotAscendingOrder(decoded));
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
            if let Ok(decoded) = <StateTransitionChainsNotAllowed as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::StateTransitionChainsNotAllowed(decoded));
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
            if let Ok(decoded) = <UpdateToOldRollupTypeID as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::UpdateToOldRollupTypeID(decoded));
            }
            if let Ok(decoded) = <UpdateToSameRollupTypeID as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::UpdateToSameRollupTypeID(decoded));
            }
            if let Ok(decoded) = <zkGasPriceOfRange as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::zkGasPriceOfRange(decoded));
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
                Self::AllBatchesMustBeVerified(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::AllSequencedMustBeVerified(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::AllzkEVMSequencedBatchesMustBeVerified(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::BatchFeeOutOfRange(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CannotUpdateWithUnconsolidatedPendingState(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ChainIDAlreadyExist(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ChainIDOutOfRange(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::EmptyVerifySequencesData(element) => {
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
                Self::FinalNumSequenceBelowLastVerifiedSequence(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::FinalNumSequenceDoesNotMatchPendingState(element) => {
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
                Self::InitSequenceMustMatchCurrentForkID(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InitSequenceNumDoesNotMatchPendingState(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidPessimisticProof(element) => {
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
                Self::InvalidRangeMultiplierZkGasPrice(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidRangeSequenceTimeTarget(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidRollup(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidRollupType(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidVerifierType(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::L1InfoTreeLeafCountInvalid(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::MustSequenceSomeBatch(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::MustSequenceSomeBlob(element) => {
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
                Self::NotAllowedAddress(element) => {
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
                Self::OnlyRollupAdmin(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::OnlyStateTransitionChains(element) => {
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
                Self::PendingStateNumExist(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ReentrancyGuardReentrantCall(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RollbackBatchIsNotEndOfSequence(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RollbackBatchIsNotValid(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RollupAddressAlreadyExist(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RollupIDNotAscendingOrder(element) => {
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
                Self::StateTransitionChainsNotAllowed(element) => {
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
                Self::UpdateToOldRollupTypeID(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::UpdateToSameRollupTypeID(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::zkGasPriceOfRange(element) => {
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
                    == <AllBatchesMustBeVerified as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <AllSequencedMustBeVerified as ::ethers::contract::EthError>::selector() => {
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
                    == <CannotUpdateWithUnconsolidatedPendingState as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <ChainIDAlreadyExist as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <ChainIDOutOfRange as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <EmptyVerifySequencesData as ::ethers::contract::EthError>::selector() => {
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
                    == <FinalNumSequenceBelowLastVerifiedSequence as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <FinalNumSequenceDoesNotMatchPendingState as ::ethers::contract::EthError>::selector() => {
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
                    == <InitSequenceMustMatchCurrentForkID as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InitSequenceNumDoesNotMatchPendingState as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InvalidPessimisticProof as ::ethers::contract::EthError>::selector() => {
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
                    == <InvalidRangeMultiplierZkGasPrice as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InvalidRangeSequenceTimeTarget as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InvalidRollup as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InvalidRollupType as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InvalidVerifierType as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <L1InfoTreeLeafCountInvalid as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <MustSequenceSomeBatch as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <MustSequenceSomeBlob as ::ethers::contract::EthError>::selector() => {
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
                    == <NotAllowedAddress as ::ethers::contract::EthError>::selector() => {
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
                    == <OnlyRollupAdmin as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <OnlyStateTransitionChains as ::ethers::contract::EthError>::selector() => {
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
                    == <PendingStateNumExist as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <ReentrancyGuardReentrantCall as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <RollbackBatchIsNotEndOfSequence as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <RollbackBatchIsNotValid as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <RollupAddressAlreadyExist as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <RollupIDNotAscendingOrder as ::ethers::contract::EthError>::selector() => {
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
                    == <StateTransitionChainsNotAllowed as ::ethers::contract::EthError>::selector() => {
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
                    == <UpdateToOldRollupTypeID as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <UpdateToSameRollupTypeID as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <zkGasPriceOfRange as ::ethers::contract::EthError>::selector() => {
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
                Self::AllBatchesMustBeVerified(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::AllSequencedMustBeVerified(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::AllzkEVMSequencedBatchesMustBeVerified(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::BatchFeeOutOfRange(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::CannotUpdateWithUnconsolidatedPendingState(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ChainIDAlreadyExist(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ChainIDOutOfRange(element) => ::core::fmt::Display::fmt(element, f),
                Self::EmptyVerifySequencesData(element) => {
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
                Self::FinalNumSequenceBelowLastVerifiedSequence(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::FinalNumSequenceDoesNotMatchPendingState(element) => {
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
                Self::InitSequenceMustMatchCurrentForkID(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InitSequenceNumDoesNotMatchPendingState(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InvalidPessimisticProof(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InvalidProof(element) => ::core::fmt::Display::fmt(element, f),
                Self::InvalidRangeBatchTimeTarget(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InvalidRangeMultiplierBatchFee(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InvalidRangeMultiplierZkGasPrice(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InvalidRangeSequenceTimeTarget(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InvalidRollup(element) => ::core::fmt::Display::fmt(element, f),
                Self::InvalidRollupType(element) => ::core::fmt::Display::fmt(element, f),
                Self::InvalidVerifierType(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::L1InfoTreeLeafCountInvalid(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::MustSequenceSomeBatch(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::MustSequenceSomeBlob(element) => {
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
                Self::NotAllowedAddress(element) => ::core::fmt::Display::fmt(element, f),
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
                Self::OnlyRollupAdmin(element) => ::core::fmt::Display::fmt(element, f),
                Self::OnlyStateTransitionChains(element) => {
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
                Self::PendingStateNumExist(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ReentrancyGuardReentrantCall(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RollbackBatchIsNotEndOfSequence(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RollbackBatchIsNotValid(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RollupAddressAlreadyExist(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RollupIDNotAscendingOrder(element) => {
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
                Self::StateTransitionChainsNotAllowed(element) => {
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
                Self::UpdateToOldRollupTypeID(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::UpdateToSameRollupTypeID(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::zkGasPriceOfRange(element) => ::core::fmt::Display::fmt(element, f),
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
    impl ::core::convert::From<AllBatchesMustBeVerified> for PolygonRollupManagerErrors {
        fn from(value: AllBatchesMustBeVerified) -> Self {
            Self::AllBatchesMustBeVerified(value)
        }
    }
    impl ::core::convert::From<AllSequencedMustBeVerified>
    for PolygonRollupManagerErrors {
        fn from(value: AllSequencedMustBeVerified) -> Self {
            Self::AllSequencedMustBeVerified(value)
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
    impl ::core::convert::From<CannotUpdateWithUnconsolidatedPendingState>
    for PolygonRollupManagerErrors {
        fn from(value: CannotUpdateWithUnconsolidatedPendingState) -> Self {
            Self::CannotUpdateWithUnconsolidatedPendingState(value)
        }
    }
    impl ::core::convert::From<ChainIDAlreadyExist> for PolygonRollupManagerErrors {
        fn from(value: ChainIDAlreadyExist) -> Self {
            Self::ChainIDAlreadyExist(value)
        }
    }
    impl ::core::convert::From<ChainIDOutOfRange> for PolygonRollupManagerErrors {
        fn from(value: ChainIDOutOfRange) -> Self {
            Self::ChainIDOutOfRange(value)
        }
    }
    impl ::core::convert::From<EmptyVerifySequencesData> for PolygonRollupManagerErrors {
        fn from(value: EmptyVerifySequencesData) -> Self {
            Self::EmptyVerifySequencesData(value)
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
    impl ::core::convert::From<FinalNumSequenceBelowLastVerifiedSequence>
    for PolygonRollupManagerErrors {
        fn from(value: FinalNumSequenceBelowLastVerifiedSequence) -> Self {
            Self::FinalNumSequenceBelowLastVerifiedSequence(value)
        }
    }
    impl ::core::convert::From<FinalNumSequenceDoesNotMatchPendingState>
    for PolygonRollupManagerErrors {
        fn from(value: FinalNumSequenceDoesNotMatchPendingState) -> Self {
            Self::FinalNumSequenceDoesNotMatchPendingState(value)
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
    impl ::core::convert::From<InitSequenceMustMatchCurrentForkID>
    for PolygonRollupManagerErrors {
        fn from(value: InitSequenceMustMatchCurrentForkID) -> Self {
            Self::InitSequenceMustMatchCurrentForkID(value)
        }
    }
    impl ::core::convert::From<InitSequenceNumDoesNotMatchPendingState>
    for PolygonRollupManagerErrors {
        fn from(value: InitSequenceNumDoesNotMatchPendingState) -> Self {
            Self::InitSequenceNumDoesNotMatchPendingState(value)
        }
    }
    impl ::core::convert::From<InvalidPessimisticProof> for PolygonRollupManagerErrors {
        fn from(value: InvalidPessimisticProof) -> Self {
            Self::InvalidPessimisticProof(value)
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
    impl ::core::convert::From<InvalidRangeMultiplierZkGasPrice>
    for PolygonRollupManagerErrors {
        fn from(value: InvalidRangeMultiplierZkGasPrice) -> Self {
            Self::InvalidRangeMultiplierZkGasPrice(value)
        }
    }
    impl ::core::convert::From<InvalidRangeSequenceTimeTarget>
    for PolygonRollupManagerErrors {
        fn from(value: InvalidRangeSequenceTimeTarget) -> Self {
            Self::InvalidRangeSequenceTimeTarget(value)
        }
    }
    impl ::core::convert::From<InvalidRollup> for PolygonRollupManagerErrors {
        fn from(value: InvalidRollup) -> Self {
            Self::InvalidRollup(value)
        }
    }
    impl ::core::convert::From<InvalidRollupType> for PolygonRollupManagerErrors {
        fn from(value: InvalidRollupType) -> Self {
            Self::InvalidRollupType(value)
        }
    }
    impl ::core::convert::From<InvalidVerifierType> for PolygonRollupManagerErrors {
        fn from(value: InvalidVerifierType) -> Self {
            Self::InvalidVerifierType(value)
        }
    }
    impl ::core::convert::From<L1InfoTreeLeafCountInvalid>
    for PolygonRollupManagerErrors {
        fn from(value: L1InfoTreeLeafCountInvalid) -> Self {
            Self::L1InfoTreeLeafCountInvalid(value)
        }
    }
    impl ::core::convert::From<MustSequenceSomeBatch> for PolygonRollupManagerErrors {
        fn from(value: MustSequenceSomeBatch) -> Self {
            Self::MustSequenceSomeBatch(value)
        }
    }
    impl ::core::convert::From<MustSequenceSomeBlob> for PolygonRollupManagerErrors {
        fn from(value: MustSequenceSomeBlob) -> Self {
            Self::MustSequenceSomeBlob(value)
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
    impl ::core::convert::From<NotAllowedAddress> for PolygonRollupManagerErrors {
        fn from(value: NotAllowedAddress) -> Self {
            Self::NotAllowedAddress(value)
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
    impl ::core::convert::From<OnlyRollupAdmin> for PolygonRollupManagerErrors {
        fn from(value: OnlyRollupAdmin) -> Self {
            Self::OnlyRollupAdmin(value)
        }
    }
    impl ::core::convert::From<OnlyStateTransitionChains>
    for PolygonRollupManagerErrors {
        fn from(value: OnlyStateTransitionChains) -> Self {
            Self::OnlyStateTransitionChains(value)
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
    impl ::core::convert::From<PendingStateNumExist> for PolygonRollupManagerErrors {
        fn from(value: PendingStateNumExist) -> Self {
            Self::PendingStateNumExist(value)
        }
    }
    impl ::core::convert::From<ReentrancyGuardReentrantCall>
    for PolygonRollupManagerErrors {
        fn from(value: ReentrancyGuardReentrantCall) -> Self {
            Self::ReentrancyGuardReentrantCall(value)
        }
    }
    impl ::core::convert::From<RollbackBatchIsNotEndOfSequence>
    for PolygonRollupManagerErrors {
        fn from(value: RollbackBatchIsNotEndOfSequence) -> Self {
            Self::RollbackBatchIsNotEndOfSequence(value)
        }
    }
    impl ::core::convert::From<RollbackBatchIsNotValid> for PolygonRollupManagerErrors {
        fn from(value: RollbackBatchIsNotValid) -> Self {
            Self::RollbackBatchIsNotValid(value)
        }
    }
    impl ::core::convert::From<RollupAddressAlreadyExist>
    for PolygonRollupManagerErrors {
        fn from(value: RollupAddressAlreadyExist) -> Self {
            Self::RollupAddressAlreadyExist(value)
        }
    }
    impl ::core::convert::From<RollupIDNotAscendingOrder>
    for PolygonRollupManagerErrors {
        fn from(value: RollupIDNotAscendingOrder) -> Self {
            Self::RollupIDNotAscendingOrder(value)
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
    impl ::core::convert::From<StateTransitionChainsNotAllowed>
    for PolygonRollupManagerErrors {
        fn from(value: StateTransitionChainsNotAllowed) -> Self {
            Self::StateTransitionChainsNotAllowed(value)
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
    impl ::core::convert::From<UpdateToOldRollupTypeID> for PolygonRollupManagerErrors {
        fn from(value: UpdateToOldRollupTypeID) -> Self {
            Self::UpdateToOldRollupTypeID(value)
        }
    }
    impl ::core::convert::From<UpdateToSameRollupTypeID> for PolygonRollupManagerErrors {
        fn from(value: UpdateToSameRollupTypeID) -> Self {
            Self::UpdateToSameRollupTypeID(value)
        }
    }
    impl ::core::convert::From<zkGasPriceOfRange> for PolygonRollupManagerErrors {
        fn from(value: zkGasPriceOfRange) -> Self {
            Self::zkGasPriceOfRange(value)
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
        abi = "AddExistingRollup(uint32,uint64,address,uint64,uint8,uint64,bytes32,bytes32)"
    )]
    pub struct AddExistingRollupFilter {
        #[ethevent(indexed)]
        pub rollup_id: u32,
        pub fork_id: u64,
        pub rollup_address: ::ethers::core::types::Address,
        pub chain_id: u64,
        pub rollup_verifier_type: u8,
        pub last_verified_batch_before_upgrade: u64,
        pub program_v_key: [u8; 32],
        pub init_pessimistic_root: [u8; 32],
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
        abi = "AddNewRollupType(uint32,address,address,uint64,uint8,bytes32,string,bytes32)"
    )]
    pub struct AddNewRollupTypeFilter {
        #[ethevent(indexed)]
        pub rollup_type_id: u32,
        pub consensus_implementation: ::ethers::core::types::Address,
        pub verifier: ::ethers::core::types::Address,
        pub fork_id: u64,
        pub rollup_verifier_type: u8,
        pub genesis: [u8; 32],
        pub description: ::std::string::String,
        pub program_v_key: [u8; 32],
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
    #[ethevent(name = "RollbackBatches", abi = "RollbackBatches(uint32,uint64,bytes32)")]
    pub struct RollbackBatchesFilter {
        #[ethevent(indexed)]
        pub rollup_id: u32,
        #[ethevent(indexed)]
        pub target_batch: u64,
        pub acc_input_hash_to_rollback: [u8; 32],
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
        name = "UpdateRollupManagerVersion",
        abi = "UpdateRollupManagerVersion(string)"
    )]
    pub struct UpdateRollupManagerVersionFilter {
        pub rollup_manager_version: ::std::string::String,
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
        CreateNewRollupFilter(CreateNewRollupFilter),
        EmergencyStateActivatedFilter(EmergencyStateActivatedFilter),
        EmergencyStateDeactivatedFilter(EmergencyStateDeactivatedFilter),
        InitializedFilter(InitializedFilter),
        ObsoleteRollupTypeFilter(ObsoleteRollupTypeFilter),
        OnSequenceBatchesFilter(OnSequenceBatchesFilter),
        RoleAdminChangedFilter(RoleAdminChangedFilter),
        RoleGrantedFilter(RoleGrantedFilter),
        RoleRevokedFilter(RoleRevokedFilter),
        RollbackBatchesFilter(RollbackBatchesFilter),
        SetBatchFeeFilter(SetBatchFeeFilter),
        SetTrustedAggregatorFilter(SetTrustedAggregatorFilter),
        UpdateRollupFilter(UpdateRollupFilter),
        UpdateRollupManagerVersionFilter(UpdateRollupManagerVersionFilter),
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
            if let Ok(decoded) = RoleAdminChangedFilter::decode_log(log) {
                return Ok(PolygonRollupManagerEvents::RoleAdminChangedFilter(decoded));
            }
            if let Ok(decoded) = RoleGrantedFilter::decode_log(log) {
                return Ok(PolygonRollupManagerEvents::RoleGrantedFilter(decoded));
            }
            if let Ok(decoded) = RoleRevokedFilter::decode_log(log) {
                return Ok(PolygonRollupManagerEvents::RoleRevokedFilter(decoded));
            }
            if let Ok(decoded) = RollbackBatchesFilter::decode_log(log) {
                return Ok(PolygonRollupManagerEvents::RollbackBatchesFilter(decoded));
            }
            if let Ok(decoded) = SetBatchFeeFilter::decode_log(log) {
                return Ok(PolygonRollupManagerEvents::SetBatchFeeFilter(decoded));
            }
            if let Ok(decoded) = SetTrustedAggregatorFilter::decode_log(log) {
                return Ok(
                    PolygonRollupManagerEvents::SetTrustedAggregatorFilter(decoded),
                );
            }
            if let Ok(decoded) = UpdateRollupFilter::decode_log(log) {
                return Ok(PolygonRollupManagerEvents::UpdateRollupFilter(decoded));
            }
            if let Ok(decoded) = UpdateRollupManagerVersionFilter::decode_log(log) {
                return Ok(
                    PolygonRollupManagerEvents::UpdateRollupManagerVersionFilter(decoded),
                );
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
                Self::RoleAdminChangedFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RoleGrantedFilter(element) => ::core::fmt::Display::fmt(element, f),
                Self::RoleRevokedFilter(element) => ::core::fmt::Display::fmt(element, f),
                Self::RollbackBatchesFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::SetBatchFeeFilter(element) => ::core::fmt::Display::fmt(element, f),
                Self::SetTrustedAggregatorFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::UpdateRollupFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::UpdateRollupManagerVersionFilter(element) => {
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
    impl ::core::convert::From<RollbackBatchesFilter> for PolygonRollupManagerEvents {
        fn from(value: RollbackBatchesFilter) -> Self {
            Self::RollbackBatchesFilter(value)
        }
    }
    impl ::core::convert::From<SetBatchFeeFilter> for PolygonRollupManagerEvents {
        fn from(value: SetBatchFeeFilter) -> Self {
            Self::SetBatchFeeFilter(value)
        }
    }
    impl ::core::convert::From<SetTrustedAggregatorFilter>
    for PolygonRollupManagerEvents {
        fn from(value: SetTrustedAggregatorFilter) -> Self {
            Self::SetTrustedAggregatorFilter(value)
        }
    }
    impl ::core::convert::From<UpdateRollupFilter> for PolygonRollupManagerEvents {
        fn from(value: UpdateRollupFilter) -> Self {
            Self::UpdateRollupFilter(value)
        }
    }
    impl ::core::convert::From<UpdateRollupManagerVersionFilter>
    for PolygonRollupManagerEvents {
        fn from(value: UpdateRollupManagerVersionFilter) -> Self {
            Self::UpdateRollupManagerVersionFilter(value)
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
    ///Container type for all input parameters for the `ROLLUP_MANAGER_VERSION` function with signature `ROLLUP_MANAGER_VERSION()` and selector `0xd8905812`
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
    #[ethcall(name = "ROLLUP_MANAGER_VERSION", abi = "ROLLUP_MANAGER_VERSION()")]
    pub struct RollupManagerVersionCall;
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
    ///Container type for all input parameters for the `addExistingRollup` function with signature `addExistingRollup(address,address,uint64,uint64,bytes32,uint8,bytes32,bytes32)` and selector `0xe80e5030`
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
        abi = "addExistingRollup(address,address,uint64,uint64,bytes32,uint8,bytes32,bytes32)"
    )]
    pub struct AddExistingRollupCall {
        pub rollup_address: ::ethers::core::types::Address,
        pub verifier: ::ethers::core::types::Address,
        pub fork_id: u64,
        pub chain_id: u64,
        pub init_root: [u8; 32],
        pub rollup_verifier_type: u8,
        pub program_v_key: [u8; 32],
        pub init_pessimistic_root: [u8; 32],
    }
    ///Container type for all input parameters for the `addNewRollupType` function with signature `addNewRollupType(address,address,uint64,uint8,bytes32,string,bytes32)` and selector `0xabcb5198`
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
        abi = "addNewRollupType(address,address,uint64,uint8,bytes32,string,bytes32)"
    )]
    pub struct AddNewRollupTypeCall {
        pub consensus_implementation: ::ethers::core::types::Address,
        pub verifier: ::ethers::core::types::Address,
        pub fork_id: u64,
        pub rollup_verifier_type: u8,
        pub genesis: [u8; 32],
        pub description: ::std::string::String,
        pub program_v_key: [u8; 32],
    }
    ///Container type for all input parameters for the `aggLayerGateway` function with signature `aggLayerGateway()` and selector `0xab0475cf`
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
    #[ethcall(name = "aggLayerGateway", abi = "aggLayerGateway()")]
    pub struct AggLayerGatewayCall;
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
    ///Container type for all input parameters for the `createNewRollup` function with signature `createNewRollup(uint32,uint64,address,address,address,string,string,bytes)` and selector `0xc5b4fdb6`
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
        abi = "createNewRollup(uint32,uint64,address,address,address,string,string,bytes)"
    )]
    pub struct CreateNewRollupCall {
        pub rollup_type_id: u32,
        pub chain_id: u64,
        pub admin: ::ethers::core::types::Address,
        pub sequencer: ::ethers::core::types::Address,
        pub gas_token_address: ::ethers::core::types::Address,
        pub sequencer_url: ::std::string::String,
        pub network_name: ::std::string::String,
        pub initialize_bytes_custom_chain: ::ethers::core::types::Bytes,
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
    ///Container type for all input parameters for the `getInputPessimisticBytes` function with signature `getInputPessimisticBytes(uint32,bytes32,bytes32,bytes32,bytes)` and selector `0xdd0464b9`
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
        name = "getInputPessimisticBytes",
        abi = "getInputPessimisticBytes(uint32,bytes32,bytes32,bytes32,bytes)"
    )]
    pub struct GetInputPessimisticBytesCall {
        pub rollup_id: u32,
        pub l_1_info_tree_root: [u8; 32],
        pub new_local_exit_root: [u8; 32],
        pub new_pessimistic_root: [u8; 32],
        pub custom_chain_data: ::ethers::core::types::Bytes,
    }
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
    ///Container type for all input parameters for the `initialize` function with signature `initialize()` and selector `0x8129fc1c`
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
    #[ethcall(name = "initialize", abi = "initialize()")]
    pub struct InitializeCall;
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
    ///Container type for all input parameters for the `rollbackBatches` function with signature `rollbackBatches(address,uint64)` and selector `0x8fd88cc2`
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
    #[ethcall(name = "rollbackBatches", abi = "rollbackBatches(address,uint64)")]
    pub struct RollbackBatchesCall {
        pub rollup_contract: ::ethers::core::types::Address,
        pub target_batch: u64,
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
    ///Container type for all input parameters for the `rollupIDToRollupDataDeserialized` function with signature `rollupIDToRollupDataDeserialized(uint32)` and selector `0xe4f3d8f9`
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
        name = "rollupIDToRollupDataDeserialized",
        abi = "rollupIDToRollupDataDeserialized(uint32)"
    )]
    pub struct RollupIDToRollupDataDeserializedCall {
        pub rollup_id: u32,
    }
    ///Container type for all input parameters for the `rollupIDToRollupDataV2` function with signature `rollupIDToRollupDataV2(uint32)` and selector `0x74d9c244`
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
    #[ethcall(name = "rollupIDToRollupDataV2", abi = "rollupIDToRollupDataV2(uint32)")]
    pub struct RollupIDToRollupDataV2Call {
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
    ///Container type for all input parameters for the `updateRollupByRollupAdmin` function with signature `updateRollupByRollupAdmin(address,uint32)` and selector `0xdfdb8c5e`
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
        name = "updateRollupByRollupAdmin",
        abi = "updateRollupByRollupAdmin(address,uint32)"
    )]
    pub struct UpdateRollupByRollupAdminCall {
        pub rollup_contract: ::ethers::core::types::Address,
        pub new_rollup_type_id: u32,
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
    ///Container type for all input parameters for the `verifyPessimisticTrustedAggregator` function with signature `verifyPessimisticTrustedAggregator(uint32,uint32,bytes32,bytes32,bytes,bytes)` and selector `0x6c766877`
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
        name = "verifyPessimisticTrustedAggregator",
        abi = "verifyPessimisticTrustedAggregator(uint32,uint32,bytes32,bytes32,bytes,bytes)"
    )]
    pub struct VerifyPessimisticTrustedAggregatorCall {
        pub rollup_id: u32,
        pub l_1_info_tree_leaf_count: u32,
        pub new_local_exit_root: [u8; 32],
        pub new_pessimistic_root: [u8; 32],
        pub proof: ::ethers::core::types::Bytes,
        pub custom_chain_data: ::ethers::core::types::Bytes,
    }
    ///Container type for all of the contract's call
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum PolygonRollupManagerCalls {
        DefaultAdminRole(DefaultAdminRoleCall),
        RollupManagerVersion(RollupManagerVersionCall),
        ActivateEmergencyState(ActivateEmergencyStateCall),
        AddExistingRollup(AddExistingRollupCall),
        AddNewRollupType(AddNewRollupTypeCall),
        AggLayerGateway(AggLayerGatewayCall),
        BridgeAddress(BridgeAddressCall),
        CalculateRewardPerBatch(CalculateRewardPerBatchCall),
        ChainIDToRollupID(ChainIDToRollupIDCall),
        CreateNewRollup(CreateNewRollupCall),
        DeactivateEmergencyState(DeactivateEmergencyStateCall),
        GetBatchFee(GetBatchFeeCall),
        GetForcedBatchFee(GetForcedBatchFeeCall),
        GetInputPessimisticBytes(GetInputPessimisticBytesCall),
        GetInputSnarkBytes(GetInputSnarkBytesCall),
        GetLastVerifiedBatch(GetLastVerifiedBatchCall),
        GetRoleAdmin(GetRoleAdminCall),
        GetRollupBatchNumToStateRoot(GetRollupBatchNumToStateRootCall),
        GetRollupExitRoot(GetRollupExitRootCall),
        GetRollupSequencedBatches(GetRollupSequencedBatchesCall),
        GlobalExitRootManager(GlobalExitRootManagerCall),
        GrantRole(GrantRoleCall),
        HasRole(HasRoleCall),
        Initialize(InitializeCall),
        IsEmergencyState(IsEmergencyStateCall),
        LastAggregationTimestamp(LastAggregationTimestampCall),
        LastDeactivatedEmergencyStateTimestamp(
            LastDeactivatedEmergencyStateTimestampCall,
        ),
        ObsoleteRollupType(ObsoleteRollupTypeCall),
        OnSequenceBatches(OnSequenceBatchesCall),
        Pol(PolCall),
        RenounceRole(RenounceRoleCall),
        RevokeRole(RevokeRoleCall),
        RollbackBatches(RollbackBatchesCall),
        RollupAddressToID(RollupAddressToIDCall),
        RollupCount(RollupCountCall),
        RollupIDToRollupData(RollupIDToRollupDataCall),
        RollupIDToRollupDataDeserialized(RollupIDToRollupDataDeserializedCall),
        RollupIDToRollupDataV2(RollupIDToRollupDataV2Call),
        RollupTypeCount(RollupTypeCountCall),
        RollupTypeMap(RollupTypeMapCall),
        SetBatchFee(SetBatchFeeCall),
        TotalSequencedBatches(TotalSequencedBatchesCall),
        TotalVerifiedBatches(TotalVerifiedBatchesCall),
        UpdateRollup(UpdateRollupCall),
        UpdateRollupByRollupAdmin(UpdateRollupByRollupAdminCall),
        VerifyBatchesTrustedAggregator(VerifyBatchesTrustedAggregatorCall),
        VerifyPessimisticTrustedAggregator(VerifyPessimisticTrustedAggregatorCall),
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
            if let Ok(decoded) = <RollupManagerVersionCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::RollupManagerVersion(decoded));
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
            if let Ok(decoded) = <AggLayerGatewayCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::AggLayerGateway(decoded));
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
            if let Ok(decoded) = <GetInputPessimisticBytesCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GetInputPessimisticBytes(decoded));
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
            if let Ok(decoded) = <PolCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::Pol(decoded));
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
            if let Ok(decoded) = <RollbackBatchesCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::RollbackBatches(decoded));
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
            if let Ok(decoded) = <RollupIDToRollupDataDeserializedCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::RollupIDToRollupDataDeserialized(decoded));
            }
            if let Ok(decoded) = <RollupIDToRollupDataV2Call as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::RollupIDToRollupDataV2(decoded));
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
            if let Ok(decoded) = <UpdateRollupCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::UpdateRollup(decoded));
            }
            if let Ok(decoded) = <UpdateRollupByRollupAdminCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::UpdateRollupByRollupAdmin(decoded));
            }
            if let Ok(decoded) = <VerifyBatchesTrustedAggregatorCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::VerifyBatchesTrustedAggregator(decoded));
            }
            if let Ok(decoded) = <VerifyPessimisticTrustedAggregatorCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::VerifyPessimisticTrustedAggregator(decoded));
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
                Self::RollupManagerVersion(element) => {
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
                Self::AggLayerGateway(element) => {
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
                Self::GetInputPessimisticBytes(element) => {
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
                Self::LastAggregationTimestamp(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::LastDeactivatedEmergencyStateTimestamp(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ObsoleteRollupType(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::OnSequenceBatches(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::Pol(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::RenounceRole(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RevokeRole(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RollbackBatches(element) => {
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
                Self::RollupIDToRollupDataDeserialized(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RollupIDToRollupDataV2(element) => {
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
                Self::TotalSequencedBatches(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::TotalVerifiedBatches(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::UpdateRollup(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::UpdateRollupByRollupAdmin(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::VerifyBatchesTrustedAggregator(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::VerifyPessimisticTrustedAggregator(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
            }
        }
    }
    impl ::core::fmt::Display for PolygonRollupManagerCalls {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::DefaultAdminRole(element) => ::core::fmt::Display::fmt(element, f),
                Self::RollupManagerVersion(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ActivateEmergencyState(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::AddExistingRollup(element) => ::core::fmt::Display::fmt(element, f),
                Self::AddNewRollupType(element) => ::core::fmt::Display::fmt(element, f),
                Self::AggLayerGateway(element) => ::core::fmt::Display::fmt(element, f),
                Self::BridgeAddress(element) => ::core::fmt::Display::fmt(element, f),
                Self::CalculateRewardPerBatch(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ChainIDToRollupID(element) => ::core::fmt::Display::fmt(element, f),
                Self::CreateNewRollup(element) => ::core::fmt::Display::fmt(element, f),
                Self::DeactivateEmergencyState(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::GetBatchFee(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetForcedBatchFee(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetInputPessimisticBytes(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
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
                Self::LastAggregationTimestamp(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::LastDeactivatedEmergencyStateTimestamp(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ObsoleteRollupType(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::OnSequenceBatches(element) => ::core::fmt::Display::fmt(element, f),
                Self::Pol(element) => ::core::fmt::Display::fmt(element, f),
                Self::RenounceRole(element) => ::core::fmt::Display::fmt(element, f),
                Self::RevokeRole(element) => ::core::fmt::Display::fmt(element, f),
                Self::RollbackBatches(element) => ::core::fmt::Display::fmt(element, f),
                Self::RollupAddressToID(element) => ::core::fmt::Display::fmt(element, f),
                Self::RollupCount(element) => ::core::fmt::Display::fmt(element, f),
                Self::RollupIDToRollupData(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RollupIDToRollupDataDeserialized(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RollupIDToRollupDataV2(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RollupTypeCount(element) => ::core::fmt::Display::fmt(element, f),
                Self::RollupTypeMap(element) => ::core::fmt::Display::fmt(element, f),
                Self::SetBatchFee(element) => ::core::fmt::Display::fmt(element, f),
                Self::TotalSequencedBatches(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::TotalVerifiedBatches(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::UpdateRollup(element) => ::core::fmt::Display::fmt(element, f),
                Self::UpdateRollupByRollupAdmin(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::VerifyBatchesTrustedAggregator(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::VerifyPessimisticTrustedAggregator(element) => {
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
    impl ::core::convert::From<RollupManagerVersionCall> for PolygonRollupManagerCalls {
        fn from(value: RollupManagerVersionCall) -> Self {
            Self::RollupManagerVersion(value)
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
    impl ::core::convert::From<AggLayerGatewayCall> for PolygonRollupManagerCalls {
        fn from(value: AggLayerGatewayCall) -> Self {
            Self::AggLayerGateway(value)
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
    impl ::core::convert::From<GetInputPessimisticBytesCall>
    for PolygonRollupManagerCalls {
        fn from(value: GetInputPessimisticBytesCall) -> Self {
            Self::GetInputPessimisticBytes(value)
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
    impl ::core::convert::From<PolCall> for PolygonRollupManagerCalls {
        fn from(value: PolCall) -> Self {
            Self::Pol(value)
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
    impl ::core::convert::From<RollbackBatchesCall> for PolygonRollupManagerCalls {
        fn from(value: RollbackBatchesCall) -> Self {
            Self::RollbackBatches(value)
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
    impl ::core::convert::From<RollupIDToRollupDataDeserializedCall>
    for PolygonRollupManagerCalls {
        fn from(value: RollupIDToRollupDataDeserializedCall) -> Self {
            Self::RollupIDToRollupDataDeserialized(value)
        }
    }
    impl ::core::convert::From<RollupIDToRollupDataV2Call>
    for PolygonRollupManagerCalls {
        fn from(value: RollupIDToRollupDataV2Call) -> Self {
            Self::RollupIDToRollupDataV2(value)
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
    impl ::core::convert::From<UpdateRollupCall> for PolygonRollupManagerCalls {
        fn from(value: UpdateRollupCall) -> Self {
            Self::UpdateRollup(value)
        }
    }
    impl ::core::convert::From<UpdateRollupByRollupAdminCall>
    for PolygonRollupManagerCalls {
        fn from(value: UpdateRollupByRollupAdminCall) -> Self {
            Self::UpdateRollupByRollupAdmin(value)
        }
    }
    impl ::core::convert::From<VerifyBatchesTrustedAggregatorCall>
    for PolygonRollupManagerCalls {
        fn from(value: VerifyBatchesTrustedAggregatorCall) -> Self {
            Self::VerifyBatchesTrustedAggregator(value)
        }
    }
    impl ::core::convert::From<VerifyPessimisticTrustedAggregatorCall>
    for PolygonRollupManagerCalls {
        fn from(value: VerifyPessimisticTrustedAggregatorCall) -> Self {
            Self::VerifyPessimisticTrustedAggregator(value)
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
    ///Container type for all return fields from the `ROLLUP_MANAGER_VERSION` function with signature `ROLLUP_MANAGER_VERSION()` and selector `0xd8905812`
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
    pub struct RollupManagerVersionReturn(pub ::std::string::String);
    ///Container type for all return fields from the `aggLayerGateway` function with signature `aggLayerGateway()` and selector `0xab0475cf`
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
    pub struct AggLayerGatewayReturn(pub ::ethers::core::types::Address);
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
    ///Container type for all return fields from the `getInputPessimisticBytes` function with signature `getInputPessimisticBytes(uint32,bytes32,bytes32,bytes32,bytes)` and selector `0xdd0464b9`
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
    pub struct GetInputPessimisticBytesReturn(pub ::ethers::core::types::Bytes);
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
        pub rollup_data: RollupDataReturn,
    }
    ///Container type for all return fields from the `rollupIDToRollupDataDeserialized` function with signature `rollupIDToRollupDataDeserialized(uint32)` and selector `0xe4f3d8f9`
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
    pub struct RollupIDToRollupDataDeserializedReturn {
        pub rollup_contract: ::ethers::core::types::Address,
        pub chain_id: u64,
        pub verifier: ::ethers::core::types::Address,
        pub fork_id: u64,
        pub last_local_exit_root: [u8; 32],
        pub last_batch_sequenced: u64,
        pub last_verified_batch: u64,
        pub last_verified_batch_before_upgrade: u64,
        pub rollup_type_id: u64,
        pub rollup_verifier_type: u8,
        pub last_pessimistic_root: [u8; 32],
        pub program_v_key: [u8; 32],
    }
    ///Container type for all return fields from the `rollupIDToRollupDataV2` function with signature `rollupIDToRollupDataV2(uint32)` and selector `0x74d9c244`
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
    pub struct RollupIDToRollupDataV2Return {
        pub rollup_data: RollupDataReturnV2,
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
        pub rollup_verifier_type: u8,
        pub obsolete: bool,
        pub genesis: [u8; 32],
        pub program_v_key: [u8; 32],
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
    ///`RollupDataReturn(address,uint64,address,uint64,bytes32,uint64,uint64,uint64,uint64,uint64,uint64,uint8)`
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
    pub struct RollupDataReturn {
        pub rollup_contract: ::ethers::core::types::Address,
        pub chain_id: u64,
        pub verifier: ::ethers::core::types::Address,
        pub fork_id: u64,
        pub last_local_exit_root: [u8; 32],
        pub last_batch_sequenced: u64,
        pub last_verified_batch: u64,
        pub legacy_last_pending_state: u64,
        pub legacy_last_pending_state_consolidated: u64,
        pub last_verified_batch_before_upgrade: u64,
        pub rollup_type_id: u64,
        pub rollup_verifier_type: u8,
    }
    ///`RollupDataReturnV2(address,uint64,address,uint64,bytes32,uint64,uint64,uint64,uint64,uint8,bytes32,bytes32)`
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
    pub struct RollupDataReturnV2 {
        pub rollup_contract: ::ethers::core::types::Address,
        pub chain_id: u64,
        pub verifier: ::ethers::core::types::Address,
        pub fork_id: u64,
        pub last_local_exit_root: [u8; 32],
        pub last_batch_sequenced: u64,
        pub last_verified_batch: u64,
        pub last_verified_batch_before_upgrade: u64,
        pub rollup_type_id: u64,
        pub rollup_verifier_type: u8,
        pub last_pessimistic_root: [u8; 32],
        pub program_v_key: [u8; 32],
    }
}
