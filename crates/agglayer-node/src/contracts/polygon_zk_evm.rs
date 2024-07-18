pub use polygon_zk_evm::*;
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
pub mod polygon_zk_evm {
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
                                "contract IPolygonZkEVMBridgeV2",
                            ),
                        ),
                    },
                    ::ethers::core::abi::ethabi::Param {
                        name: ::std::borrow::ToOwned::to_owned("_rollupManager"),
                        kind: ::ethers::core::abi::ethabi::ParamType::Address,
                        internal_type: ::core::option::Option::Some(
                            ::std::borrow::ToOwned::to_owned(
                                "contract PolygonRollupManager",
                            ),
                        ),
                    },
                ],
            }),
            functions: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("GLOBAL_EXIT_ROOT_MANAGER_L2"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "GLOBAL_EXIT_ROOT_MANAGER_L2",
                            ),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "contract IBasePolygonZkEVMGlobalExitRoot",
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
                        "INITIALIZE_TX_BRIDGE_LIST_LEN_LEN",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "INITIALIZE_TX_BRIDGE_LIST_LEN_LEN",
                            ),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
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
                    ::std::borrow::ToOwned::to_owned("INITIALIZE_TX_BRIDGE_PARAMS"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "INITIALIZE_TX_BRIDGE_PARAMS",
                            ),
                            inputs: ::std::vec![],
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
                    ::std::borrow::ToOwned::to_owned(
                        "INITIALIZE_TX_BRIDGE_PARAMS_AFTER_BRIDGE_ADDRESS",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "INITIALIZE_TX_BRIDGE_PARAMS_AFTER_BRIDGE_ADDRESS",
                            ),
                            inputs: ::std::vec![],
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
                    ::std::borrow::ToOwned::to_owned(
                        "INITIALIZE_TX_BRIDGE_PARAMS_AFTER_BRIDGE_ADDRESS_EMPTY_METADATA",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "INITIALIZE_TX_BRIDGE_PARAMS_AFTER_BRIDGE_ADDRESS_EMPTY_METADATA",
                            ),
                            inputs: ::std::vec![],
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
                    ::std::borrow::ToOwned::to_owned("INITIALIZE_TX_CONSTANT_BYTES"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "INITIALIZE_TX_CONSTANT_BYTES",
                            ),
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
                    ::std::borrow::ToOwned::to_owned(
                        "INITIALIZE_TX_CONSTANT_BYTES_EMPTY_METADATA",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "INITIALIZE_TX_CONSTANT_BYTES_EMPTY_METADATA",
                            ),
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
                    ::std::borrow::ToOwned::to_owned(
                        "INITIALIZE_TX_DATA_LEN_EMPTY_METADATA",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "INITIALIZE_TX_DATA_LEN_EMPTY_METADATA",
                            ),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
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
                    ::std::borrow::ToOwned::to_owned(
                        "INITIALIZE_TX_EFFECTIVE_PERCENTAGE",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "INITIALIZE_TX_EFFECTIVE_PERCENTAGE",
                            ),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        1usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes1"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("SIGNATURE_INITIALIZE_TX_R"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "SIGNATURE_INITIALIZE_TX_R",
                            ),
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
                    ::std::borrow::ToOwned::to_owned("SIGNATURE_INITIALIZE_TX_S"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "SIGNATURE_INITIALIZE_TX_S",
                            ),
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
                    ::std::borrow::ToOwned::to_owned("SIGNATURE_INITIALIZE_TX_V"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "SIGNATURE_INITIALIZE_TX_V",
                            ),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
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
                    ::std::borrow::ToOwned::to_owned("TIMESTAMP_RANGE"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("TIMESTAMP_RANGE"),
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
                    ::std::borrow::ToOwned::to_owned("acceptAdminRole"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("acceptAdminRole"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("admin"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("admin"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
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
                                            "contract IPolygonZkEVMBridgeV2",
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
                    ::std::borrow::ToOwned::to_owned("calculatePolPerForceBatch"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "calculatePolPerForceBatch",
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
                    ::std::borrow::ToOwned::to_owned("forceBatch"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("forceBatch"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("transactions"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("polAmount"),
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
                    ::std::borrow::ToOwned::to_owned("forceBatchAddress"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("forceBatchAddress"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("forceBatchTimeout"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("forceBatchTimeout"),
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
                    ::std::borrow::ToOwned::to_owned("forcedBatches"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("forcedBatches"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
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
                    ::std::borrow::ToOwned::to_owned("gasTokenAddress"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("gasTokenAddress"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("gasTokenNetwork"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("gasTokenNetwork"),
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
                    ::std::borrow::ToOwned::to_owned("generateInitializeTransaction"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "generateInitializeTransaction",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("networkID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("_gasTokenAddress"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("_gasTokenNetwork"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("_gasTokenMetadata"),
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
                    ::std::borrow::ToOwned::to_owned("initialize"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("initialize"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("_admin"),
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
                                    name: ::std::borrow::ToOwned::to_owned("networkID"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("_gasTokenAddress"),
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
                                    name: ::std::borrow::ToOwned::to_owned("_networkName"),
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
                    ::std::borrow::ToOwned::to_owned("lastAccInputHash"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("lastAccInputHash"),
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
                    ::std::borrow::ToOwned::to_owned("lastForceBatch"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("lastForceBatch"),
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
                    ::std::borrow::ToOwned::to_owned("lastForceBatchSequenced"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "lastForceBatchSequenced",
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
                    ::std::borrow::ToOwned::to_owned("networkName"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("networkName"),
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
                    ::std::borrow::ToOwned::to_owned("onVerifyBatches"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("onVerifyBatches"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("lastVerifiedBatch"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
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
                                    name: ::std::borrow::ToOwned::to_owned("aggregator"),
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
                    ::std::borrow::ToOwned::to_owned("pendingAdmin"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("pendingAdmin"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
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
                    ::std::borrow::ToOwned::to_owned("rollupManager"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("rollupManager"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "contract PolygonRollupManager",
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
                    ::std::borrow::ToOwned::to_owned("sequenceBatches"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("sequenceBatches"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("batches"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                    ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                    ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                                ],
                                            ),
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "struct PolygonRollupBaseEtrog.BatchData[]",
                                        ),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "maxSequenceTimestamp",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "initSequencedBatch",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("l2Coinbase"),
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
                    ::std::borrow::ToOwned::to_owned("sequenceForceBatches"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "sequenceForceBatches",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("batches"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                    ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                    ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                                ],
                                            ),
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "struct PolygonRollupBaseEtrog.BatchData[]",
                                        ),
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
                    ::std::borrow::ToOwned::to_owned("setForceBatchAddress"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "setForceBatchAddress",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "newForceBatchAddress",
                                    ),
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
                    ::std::borrow::ToOwned::to_owned("setForceBatchTimeout"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "setForceBatchTimeout",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "newforceBatchTimeout",
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
                    ::std::borrow::ToOwned::to_owned("setTrustedSequencer"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "setTrustedSequencer",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "newTrustedSequencer",
                                    ),
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
                    ::std::borrow::ToOwned::to_owned("setTrustedSequencerURL"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "setTrustedSequencerURL",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "newTrustedSequencerURL",
                                    ),
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
                    ::std::borrow::ToOwned::to_owned("transferAdminRole"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("transferAdminRole"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("newPendingAdmin"),
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
                    ::std::borrow::ToOwned::to_owned("trustedSequencer"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("trustedSequencer"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("trustedSequencerURL"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "trustedSequencerURL",
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
            ]),
            events: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("AcceptAdminRole"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("AcceptAdminRole"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("newAdmin"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("ForceBatch"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("ForceBatch"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("forceBatchNum"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    indexed: true,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "lastGlobalExitRoot",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("sequencer"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("transactions"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bytes,
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("InitialSequenceBatches"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned(
                                "InitialSequenceBatches",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("transactions"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bytes,
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "lastGlobalExitRoot",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("sequencer"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: false,
                                },
                            ],
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
                    ::std::borrow::ToOwned::to_owned("SequenceBatches"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("SequenceBatches"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("numBatch"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    indexed: true,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("l1InfoRoot"),
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
                    ::std::borrow::ToOwned::to_owned("SequenceForceBatches"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned(
                                "SequenceForceBatches",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("numBatch"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    indexed: true,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("SetForceBatchAddress"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned(
                                "SetForceBatchAddress",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "newForceBatchAddress",
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
                    ::std::borrow::ToOwned::to_owned("SetForceBatchTimeout"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned(
                                "SetForceBatchTimeout",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "newforceBatchTimeout",
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
                    ::std::borrow::ToOwned::to_owned("SetTrustedSequencer"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned(
                                "SetTrustedSequencer",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "newTrustedSequencer",
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
                    ::std::borrow::ToOwned::to_owned("SetTrustedSequencerURL"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned(
                                "SetTrustedSequencerURL",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "newTrustedSequencerURL",
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
                    ::std::borrow::ToOwned::to_owned("TransferAdminRole"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("TransferAdminRole"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("newPendingAdmin"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
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
                                    name: ::std::borrow::ToOwned::to_owned("numBatch"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    indexed: true,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("stateRoot"),
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
                    ::std::borrow::ToOwned::to_owned("BatchAlreadyVerified"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "BatchAlreadyVerified",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned(
                        "BatchNotSequencedOrNotSequenceEnd",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "BatchNotSequencedOrNotSequenceEnd",
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
                    ::std::borrow::ToOwned::to_owned("ForceBatchNotAllowed"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "ForceBatchNotAllowed",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("ForceBatchTimeoutNotExpired"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "ForceBatchTimeoutNotExpired",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("ForceBatchesAlreadyActive"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "ForceBatchesAlreadyActive",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("ForceBatchesDecentralized"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "ForceBatchesDecentralized",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned(
                        "ForceBatchesNotAllowedOnEmergencyState",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "ForceBatchesNotAllowedOnEmergencyState",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("ForceBatchesOverflow"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "ForceBatchesOverflow",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("ForcedDataDoesNotMatch"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "ForcedDataDoesNotMatch",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("GasTokenNetworkMustBeZeroOnEther"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "GasTokenNetworkMustBeZeroOnEther",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("GlobalExitRootNotExist"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "GlobalExitRootNotExist",
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
                    ::std::borrow::ToOwned::to_owned(
                        "HaltTimeoutNotExpiredAfterEmergencyState",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "HaltTimeoutNotExpiredAfterEmergencyState",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("HugeTokenMetadataNotSupported"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "HugeTokenMetadataNotSupported",
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
                    ::std::borrow::ToOwned::to_owned("InitSequencedBatchDoesNotMatch"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "InitSequencedBatchDoesNotMatch",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("InvalidInitializeTransaction"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "InvalidInitializeTransaction",
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
                    ::std::borrow::ToOwned::to_owned("InvalidRangeForceBatchTimeout"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "InvalidRangeForceBatchTimeout",
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
                    ::std::borrow::ToOwned::to_owned("MaxTimestampSequenceInvalid"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "MaxTimestampSequenceInvalid",
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
                    ::std::borrow::ToOwned::to_owned("NotEnoughMaticAmount"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "NotEnoughMaticAmount",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("NotEnoughPOLAmount"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("NotEnoughPOLAmount"),
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
                    ::std::borrow::ToOwned::to_owned("OnlyAdmin"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("OnlyAdmin"),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("OnlyPendingAdmin"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("OnlyPendingAdmin"),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("OnlyRollupManager"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("OnlyRollupManager"),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("OnlyTrustedAggregator"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "OnlyTrustedAggregator",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("OnlyTrustedSequencer"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "OnlyTrustedSequencer",
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
                    ::std::borrow::ToOwned::to_owned(
                        "PendingStateTimeoutExceedHaltAggregationTimeout",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "PendingStateTimeoutExceedHaltAggregationTimeout",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("SequenceZeroBatches"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "SequenceZeroBatches",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned(
                        "SequencedTimestampBelowForcedTimestamp",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "SequencedTimestampBelowForcedTimestamp",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("SequencedTimestampInvalid"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "SequencedTimestampInvalid",
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
                    ::std::borrow::ToOwned::to_owned("TransactionsLengthAboveMax"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "TransactionsLengthAboveMax",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned(
                        "TrustedAggregatorTimeoutExceedHaltAggregationTimeout",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "TrustedAggregatorTimeoutExceedHaltAggregationTimeout",
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
            ]),
            receive: false,
            fallback: false,
        }
    }
    ///The parsed JSON ABI of the contract.
    pub static POLYGONZKEVM_ABI: ::ethers::contract::Lazy<::ethers::core::abi::Abi> = ::ethers::contract::Lazy::new(
        __abi,
    );
    pub struct PolygonZkEvm<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for PolygonZkEvm<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for PolygonZkEvm<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for PolygonZkEvm<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for PolygonZkEvm<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(::core::stringify!(PolygonZkEvm))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> PolygonZkEvm<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(
                ::ethers::contract::Contract::new(
                    address.into(),
                    POLYGONZKEVM_ABI.clone(),
                    client,
                ),
            )
        }
        ///Calls the contract's `GLOBAL_EXIT_ROOT_MANAGER_L2` (0x9e001877) function
        pub fn global_exit_root_manager_l2(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Address,
        > {
            self.0
                .method_hash([158, 0, 24, 119], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `INITIALIZE_TX_BRIDGE_LIST_LEN_LEN` (0x11e892d4) function
        pub fn initialize_tx_bridge_list_len_len(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, u8> {
            self.0
                .method_hash([17, 232, 146, 212], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `INITIALIZE_TX_BRIDGE_PARAMS` (0x05835f37) function
        pub fn initialize_tx_bridge_params(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Bytes,
        > {
            self.0
                .method_hash([5, 131, 95, 55], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `INITIALIZE_TX_BRIDGE_PARAMS_AFTER_BRIDGE_ADDRESS` (0x7a5460c5) function
        pub fn initialize_tx_bridge_params_after_bridge_address(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Bytes,
        > {
            self.0
                .method_hash([122, 84, 96, 197], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `INITIALIZE_TX_BRIDGE_PARAMS_AFTER_BRIDGE_ADDRESS_EMPTY_METADATA` (0x52bdeb6d) function
        pub fn initialize_tx_bridge_params_after_bridge_address_empty_metadata(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Bytes,
        > {
            self.0
                .method_hash([82, 189, 235, 109], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `INITIALIZE_TX_CONSTANT_BYTES` (0x03508963) function
        pub fn initialize_tx_constant_bytes(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, u16> {
            self.0
                .method_hash([3, 80, 137, 99], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `INITIALIZE_TX_CONSTANT_BYTES_EMPTY_METADATA` (0x676870d2) function
        pub fn initialize_tx_constant_bytes_empty_metadata(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, u16> {
            self.0
                .method_hash([103, 104, 112, 210], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `INITIALIZE_TX_DATA_LEN_EMPTY_METADATA` (0xc7fffd4b) function
        pub fn initialize_tx_data_len_empty_metadata(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, u8> {
            self.0
                .method_hash([199, 255, 253, 75], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `INITIALIZE_TX_EFFECTIVE_PERCENTAGE` (0x40b5de6c) function
        pub fn initialize_tx_effective_percentage(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, [u8; 1]> {
            self.0
                .method_hash([64, 181, 222, 108], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `SIGNATURE_INITIALIZE_TX_R` (0xb0afe154) function
        pub fn signature_initialize_tx_r(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([176, 175, 225, 84], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `SIGNATURE_INITIALIZE_TX_S` (0xd7bc90ff) function
        pub fn signature_initialize_tx_s(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([215, 188, 144, 255], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `SIGNATURE_INITIALIZE_TX_V` (0xf35dda47) function
        pub fn signature_initialize_tx_v(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, u8> {
            self.0
                .method_hash([243, 93, 218, 71], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `TIMESTAMP_RANGE` (0x42308fab) function
        pub fn timestamp_range(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([66, 48, 143, 171], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `acceptAdminRole` (0x8c3d7301) function
        pub fn accept_admin_role(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([140, 61, 115, 1], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `admin` (0xf851a440) function
        pub fn admin(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Address,
        > {
            self.0
                .method_hash([248, 81, 164, 64], ())
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
        ///Calls the contract's `calculatePolPerForceBatch` (0x00d0295d) function
        pub fn calculate_pol_per_force_batch(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([0, 208, 41, 93], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `forceBatch` (0xeaeb077b) function
        pub fn force_batch(
            &self,
            transactions: ::ethers::core::types::Bytes,
            pol_amount: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([234, 235, 7, 123], (transactions, pol_amount))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `forceBatchAddress` (0x2c111c06) function
        pub fn force_batch_address(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Address,
        > {
            self.0
                .method_hash([44, 17, 28, 6], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `forceBatchTimeout` (0xc754c7ed) function
        pub fn force_batch_timeout(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([199, 84, 199, 237], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `forcedBatches` (0x6b8616ce) function
        pub fn forced_batches(
            &self,
            p0: u64,
        ) -> ::ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([107, 134, 22, 206], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `gasTokenAddress` (0x3c351e10) function
        pub fn gas_token_address(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Address,
        > {
            self.0
                .method_hash([60, 53, 30, 16], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `gasTokenNetwork` (0x3cbc795b) function
        pub fn gas_token_network(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, u32> {
            self.0
                .method_hash([60, 188, 121, 91], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `generateInitializeTransaction` (0xa652f26c) function
        pub fn generate_initialize_transaction(
            &self,
            network_id: u32,
            gas_token_address: ::ethers::core::types::Address,
            gas_token_network: u32,
            gas_token_metadata: ::ethers::core::types::Bytes,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Bytes,
        > {
            self.0
                .method_hash(
                    [166, 82, 242, 108],
                    (
                        network_id,
                        gas_token_address,
                        gas_token_network,
                        gas_token_metadata,
                    ),
                )
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
        ///Calls the contract's `initialize` (0x71257022) function
        pub fn initialize(
            &self,
            admin: ::ethers::core::types::Address,
            sequencer: ::ethers::core::types::Address,
            network_id: u32,
            gas_token_address: ::ethers::core::types::Address,
            sequencer_url: ::std::string::String,
            network_name: ::std::string::String,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [113, 37, 112, 34],
                    (
                        admin,
                        sequencer,
                        network_id,
                        gas_token_address,
                        sequencer_url,
                        network_name,
                    ),
                )
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `lastAccInputHash` (0x6e05d2cd) function
        pub fn last_acc_input_hash(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([110, 5, 210, 205], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `lastForceBatch` (0xe7a7ed02) function
        pub fn last_force_batch(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([231, 167, 237, 2], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `lastForceBatchSequenced` (0x45605267) function
        pub fn last_force_batch_sequenced(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([69, 96, 82, 103], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `networkName` (0x107bf28c) function
        pub fn network_name(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::std::string::String> {
            self.0
                .method_hash([16, 123, 242, 140], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `onVerifyBatches` (0x32c2d153) function
        pub fn on_verify_batches(
            &self,
            last_verified_batch: u64,
            new_state_root: [u8; 32],
            aggregator: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [50, 194, 209, 83],
                    (last_verified_batch, new_state_root, aggregator),
                )
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `pendingAdmin` (0x26782247) function
        pub fn pending_admin(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Address,
        > {
            self.0
                .method_hash([38, 120, 34, 71], ())
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
        ///Calls the contract's `rollupManager` (0x49b7b802) function
        pub fn rollup_manager(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Address,
        > {
            self.0
                .method_hash([73, 183, 184, 2], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `sequenceBatches` (0xdef57e54) function
        pub fn sequence_batches(
            &self,
            batches: ::std::vec::Vec<BatchData>,
            max_sequence_timestamp: u64,
            init_sequenced_batch: u64,
            l_2_coinbase: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [222, 245, 126, 84],
                    (batches, max_sequence_timestamp, init_sequenced_batch, l_2_coinbase),
                )
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `sequenceForceBatches` (0x9f26f840) function
        pub fn sequence_force_batches(
            &self,
            batches: ::std::vec::Vec<BatchData>,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([159, 38, 248, 64], batches)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `setForceBatchAddress` (0x91cafe32) function
        pub fn set_force_batch_address(
            &self,
            new_force_batch_address: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([145, 202, 254, 50], new_force_batch_address)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `setForceBatchTimeout` (0x4e487706) function
        pub fn set_force_batch_timeout(
            &self,
            newforce_batch_timeout: u64,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([78, 72, 119, 6], newforce_batch_timeout)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `setTrustedSequencer` (0x6ff512cc) function
        pub fn set_trusted_sequencer(
            &self,
            new_trusted_sequencer: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([111, 245, 18, 204], new_trusted_sequencer)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `setTrustedSequencerURL` (0xc89e42df) function
        pub fn set_trusted_sequencer_url(
            &self,
            new_trusted_sequencer_url: ::std::string::String,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([200, 158, 66, 223], new_trusted_sequencer_url)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `transferAdminRole` (0xada8f919) function
        pub fn transfer_admin_role(
            &self,
            new_pending_admin: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([173, 168, 249, 25], new_pending_admin)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `trustedSequencer` (0xcfa8ed47) function
        pub fn trusted_sequencer(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Address,
        > {
            self.0
                .method_hash([207, 168, 237, 71], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `trustedSequencerURL` (0x542028d5) function
        pub fn trusted_sequencer_url(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::std::string::String> {
            self.0
                .method_hash([84, 32, 40, 213], ())
                .expect("method not found (this should never happen)")
        }
        ///Gets the contract's `AcceptAdminRole` event
        pub fn accept_admin_role_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            AcceptAdminRoleFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `ForceBatch` event
        pub fn force_batch_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            ForceBatchFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `InitialSequenceBatches` event
        pub fn initial_sequence_batches_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            InitialSequenceBatchesFilter,
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
        ///Gets the contract's `SequenceBatches` event
        pub fn sequence_batches_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            SequenceBatchesFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `SequenceForceBatches` event
        pub fn sequence_force_batches_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            SequenceForceBatchesFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `SetForceBatchAddress` event
        pub fn set_force_batch_address_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            SetForceBatchAddressFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `SetForceBatchTimeout` event
        pub fn set_force_batch_timeout_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            SetForceBatchTimeoutFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `SetTrustedSequencer` event
        pub fn set_trusted_sequencer_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            SetTrustedSequencerFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `SetTrustedSequencerURL` event
        pub fn set_trusted_sequencer_url_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            SetTrustedSequencerURLFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `TransferAdminRole` event
        pub fn transfer_admin_role_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            TransferAdminRoleFilter,
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
        /// Returns an `Event` builder for all the events of this contract.
        pub fn events(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            PolygonZkEvmEvents,
        > {
            self.0.event_with_filter(::core::default::Default::default())
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
    for PolygonZkEvm<M> {
        fn from(contract: ::ethers::contract::Contract<M>) -> Self {
            Self::new(contract.address(), contract.client())
        }
    }
    ///Custom Error type `BatchAlreadyVerified` with signature `BatchAlreadyVerified()` and selector `0x812a372d`
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
    #[etherror(name = "BatchAlreadyVerified", abi = "BatchAlreadyVerified()")]
    pub struct BatchAlreadyVerified;
    ///Custom Error type `BatchNotSequencedOrNotSequenceEnd` with signature `BatchNotSequencedOrNotSequenceEnd()` and selector `0x98c5c014`
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
        name = "BatchNotSequencedOrNotSequenceEnd",
        abi = "BatchNotSequencedOrNotSequenceEnd()"
    )]
    pub struct BatchNotSequencedOrNotSequenceEnd;
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
    ///Custom Error type `ForceBatchNotAllowed` with signature `ForceBatchNotAllowed()` and selector `0x24eff8c3`
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
    #[etherror(name = "ForceBatchNotAllowed", abi = "ForceBatchNotAllowed()")]
    pub struct ForceBatchNotAllowed;
    ///Custom Error type `ForceBatchTimeoutNotExpired` with signature `ForceBatchTimeoutNotExpired()` and selector `0xc44a0821`
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
        name = "ForceBatchTimeoutNotExpired",
        abi = "ForceBatchTimeoutNotExpired()"
    )]
    pub struct ForceBatchTimeoutNotExpired;
    ///Custom Error type `ForceBatchesAlreadyActive` with signature `ForceBatchesAlreadyActive()` and selector `0xf6ba91a1`
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
    #[etherror(name = "ForceBatchesAlreadyActive", abi = "ForceBatchesAlreadyActive()")]
    pub struct ForceBatchesAlreadyActive;
    ///Custom Error type `ForceBatchesDecentralized` with signature `ForceBatchesDecentralized()` and selector `0xc89374d8`
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
    #[etherror(name = "ForceBatchesDecentralized", abi = "ForceBatchesDecentralized()")]
    pub struct ForceBatchesDecentralized;
    ///Custom Error type `ForceBatchesNotAllowedOnEmergencyState` with signature `ForceBatchesNotAllowedOnEmergencyState()` and selector `0x39258d18`
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
        name = "ForceBatchesNotAllowedOnEmergencyState",
        abi = "ForceBatchesNotAllowedOnEmergencyState()"
    )]
    pub struct ForceBatchesNotAllowedOnEmergencyState;
    ///Custom Error type `ForceBatchesOverflow` with signature `ForceBatchesOverflow()` and selector `0xc630a00d`
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
    #[etherror(name = "ForceBatchesOverflow", abi = "ForceBatchesOverflow()")]
    pub struct ForceBatchesOverflow;
    ///Custom Error type `ForcedDataDoesNotMatch` with signature `ForcedDataDoesNotMatch()` and selector `0xce3d755e`
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
    #[etherror(name = "ForcedDataDoesNotMatch", abi = "ForcedDataDoesNotMatch()")]
    pub struct ForcedDataDoesNotMatch;
    ///Custom Error type `GasTokenNetworkMustBeZeroOnEther` with signature `GasTokenNetworkMustBeZeroOnEther()` and selector `0x1a874c12`
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
        name = "GasTokenNetworkMustBeZeroOnEther",
        abi = "GasTokenNetworkMustBeZeroOnEther()"
    )]
    pub struct GasTokenNetworkMustBeZeroOnEther;
    ///Custom Error type `GlobalExitRootNotExist` with signature `GlobalExitRootNotExist()` and selector `0x73bd668d`
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
    #[etherror(name = "GlobalExitRootNotExist", abi = "GlobalExitRootNotExist()")]
    pub struct GlobalExitRootNotExist;
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
    ///Custom Error type `HaltTimeoutNotExpiredAfterEmergencyState` with signature `HaltTimeoutNotExpiredAfterEmergencyState()` and selector `0x3d49ed4c`
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
        name = "HaltTimeoutNotExpiredAfterEmergencyState",
        abi = "HaltTimeoutNotExpiredAfterEmergencyState()"
    )]
    pub struct HaltTimeoutNotExpiredAfterEmergencyState;
    ///Custom Error type `HugeTokenMetadataNotSupported` with signature `HugeTokenMetadataNotSupported()` and selector `0x248b8f82`
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
        name = "HugeTokenMetadataNotSupported",
        abi = "HugeTokenMetadataNotSupported()"
    )]
    pub struct HugeTokenMetadataNotSupported;
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
    ///Custom Error type `InitSequencedBatchDoesNotMatch` with signature `InitSequencedBatchDoesNotMatch()` and selector `0x1a070d9a`
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
        name = "InitSequencedBatchDoesNotMatch",
        abi = "InitSequencedBatchDoesNotMatch()"
    )]
    pub struct InitSequencedBatchDoesNotMatch;
    ///Custom Error type `InvalidInitializeTransaction` with signature `InvalidInitializeTransaction()` and selector `0xcd161966`
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
        name = "InvalidInitializeTransaction",
        abi = "InvalidInitializeTransaction()"
    )]
    pub struct InvalidInitializeTransaction;
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
    ///Custom Error type `InvalidRangeForceBatchTimeout` with signature `InvalidRangeForceBatchTimeout()` and selector `0xf5e37f2f`
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
        name = "InvalidRangeForceBatchTimeout",
        abi = "InvalidRangeForceBatchTimeout()"
    )]
    pub struct InvalidRangeForceBatchTimeout;
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
    ///Custom Error type `MaxTimestampSequenceInvalid` with signature `MaxTimestampSequenceInvalid()` and selector `0x0a00feb3`
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
        name = "MaxTimestampSequenceInvalid",
        abi = "MaxTimestampSequenceInvalid()"
    )]
    pub struct MaxTimestampSequenceInvalid;
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
    ///Custom Error type `NotEnoughMaticAmount` with signature `NotEnoughMaticAmount()` and selector `0x4732fdb5`
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
    #[etherror(name = "NotEnoughMaticAmount", abi = "NotEnoughMaticAmount()")]
    pub struct NotEnoughMaticAmount;
    ///Custom Error type `NotEnoughPOLAmount` with signature `NotEnoughPOLAmount()` and selector `0x2354600f`
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
    #[etherror(name = "NotEnoughPOLAmount", abi = "NotEnoughPOLAmount()")]
    pub struct NotEnoughPOLAmount;
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
    ///Custom Error type `OnlyAdmin` with signature `OnlyAdmin()` and selector `0x47556579`
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
    #[etherror(name = "OnlyAdmin", abi = "OnlyAdmin()")]
    pub struct OnlyAdmin;
    ///Custom Error type `OnlyPendingAdmin` with signature `OnlyPendingAdmin()` and selector `0xd1ec4b23`
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
    #[etherror(name = "OnlyPendingAdmin", abi = "OnlyPendingAdmin()")]
    pub struct OnlyPendingAdmin;
    ///Custom Error type `OnlyRollupManager` with signature `OnlyRollupManager()` and selector `0xb9b3a2c8`
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
    #[etherror(name = "OnlyRollupManager", abi = "OnlyRollupManager()")]
    pub struct OnlyRollupManager;
    ///Custom Error type `OnlyTrustedAggregator` with signature `OnlyTrustedAggregator()` and selector `0xbbcbbc05`
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
    #[etherror(name = "OnlyTrustedAggregator", abi = "OnlyTrustedAggregator()")]
    pub struct OnlyTrustedAggregator;
    ///Custom Error type `OnlyTrustedSequencer` with signature `OnlyTrustedSequencer()` and selector `0x11e7be15`
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
    #[etherror(name = "OnlyTrustedSequencer", abi = "OnlyTrustedSequencer()")]
    pub struct OnlyTrustedSequencer;
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
    ///Custom Error type `PendingStateTimeoutExceedHaltAggregationTimeout` with signature `PendingStateTimeoutExceedHaltAggregationTimeout()` and selector `0xcc965070`
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
        name = "PendingStateTimeoutExceedHaltAggregationTimeout",
        abi = "PendingStateTimeoutExceedHaltAggregationTimeout()"
    )]
    pub struct PendingStateTimeoutExceedHaltAggregationTimeout;
    ///Custom Error type `SequenceZeroBatches` with signature `SequenceZeroBatches()` and selector `0xcb591a5f`
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
    #[etherror(name = "SequenceZeroBatches", abi = "SequenceZeroBatches()")]
    pub struct SequenceZeroBatches;
    ///Custom Error type `SequencedTimestampBelowForcedTimestamp` with signature `SequencedTimestampBelowForcedTimestamp()` and selector `0x7f7ab872`
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
        name = "SequencedTimestampBelowForcedTimestamp",
        abi = "SequencedTimestampBelowForcedTimestamp()"
    )]
    pub struct SequencedTimestampBelowForcedTimestamp;
    ///Custom Error type `SequencedTimestampInvalid` with signature `SequencedTimestampInvalid()` and selector `0xea827916`
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
    #[etherror(name = "SequencedTimestampInvalid", abi = "SequencedTimestampInvalid()")]
    pub struct SequencedTimestampInvalid;
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
    ///Custom Error type `TransactionsLengthAboveMax` with signature `TransactionsLengthAboveMax()` and selector `0xa29a6c7c`
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
        name = "TransactionsLengthAboveMax",
        abi = "TransactionsLengthAboveMax()"
    )]
    pub struct TransactionsLengthAboveMax;
    ///Custom Error type `TrustedAggregatorTimeoutExceedHaltAggregationTimeout` with signature `TrustedAggregatorTimeoutExceedHaltAggregationTimeout()` and selector `0x1d06e879`
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
        name = "TrustedAggregatorTimeoutExceedHaltAggregationTimeout",
        abi = "TrustedAggregatorTimeoutExceedHaltAggregationTimeout()"
    )]
    pub struct TrustedAggregatorTimeoutExceedHaltAggregationTimeout;
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
    ///Container type for all of the contract's custom errors
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum PolygonZkEvmErrors {
        BatchAlreadyVerified(BatchAlreadyVerified),
        BatchNotSequencedOrNotSequenceEnd(BatchNotSequencedOrNotSequenceEnd),
        ExceedMaxVerifyBatches(ExceedMaxVerifyBatches),
        FinalNumBatchBelowLastVerifiedBatch(FinalNumBatchBelowLastVerifiedBatch),
        FinalNumBatchDoesNotMatchPendingState(FinalNumBatchDoesNotMatchPendingState),
        FinalPendingStateNumInvalid(FinalPendingStateNumInvalid),
        ForceBatchNotAllowed(ForceBatchNotAllowed),
        ForceBatchTimeoutNotExpired(ForceBatchTimeoutNotExpired),
        ForceBatchesAlreadyActive(ForceBatchesAlreadyActive),
        ForceBatchesDecentralized(ForceBatchesDecentralized),
        ForceBatchesNotAllowedOnEmergencyState(ForceBatchesNotAllowedOnEmergencyState),
        ForceBatchesOverflow(ForceBatchesOverflow),
        ForcedDataDoesNotMatch(ForcedDataDoesNotMatch),
        GasTokenNetworkMustBeZeroOnEther(GasTokenNetworkMustBeZeroOnEther),
        GlobalExitRootNotExist(GlobalExitRootNotExist),
        HaltTimeoutNotExpired(HaltTimeoutNotExpired),
        HaltTimeoutNotExpiredAfterEmergencyState(
            HaltTimeoutNotExpiredAfterEmergencyState,
        ),
        HugeTokenMetadataNotSupported(HugeTokenMetadataNotSupported),
        InitNumBatchAboveLastVerifiedBatch(InitNumBatchAboveLastVerifiedBatch),
        InitNumBatchDoesNotMatchPendingState(InitNumBatchDoesNotMatchPendingState),
        InitSequencedBatchDoesNotMatch(InitSequencedBatchDoesNotMatch),
        InvalidInitializeTransaction(InvalidInitializeTransaction),
        InvalidProof(InvalidProof),
        InvalidRangeBatchTimeTarget(InvalidRangeBatchTimeTarget),
        InvalidRangeForceBatchTimeout(InvalidRangeForceBatchTimeout),
        InvalidRangeMultiplierBatchFee(InvalidRangeMultiplierBatchFee),
        MaxTimestampSequenceInvalid(MaxTimestampSequenceInvalid),
        NewAccInputHashDoesNotExist(NewAccInputHashDoesNotExist),
        NewPendingStateTimeoutMustBeLower(NewPendingStateTimeoutMustBeLower),
        NewStateRootNotInsidePrime(NewStateRootNotInsidePrime),
        NewTrustedAggregatorTimeoutMustBeLower(NewTrustedAggregatorTimeoutMustBeLower),
        NotEnoughMaticAmount(NotEnoughMaticAmount),
        NotEnoughPOLAmount(NotEnoughPOLAmount),
        OldAccInputHashDoesNotExist(OldAccInputHashDoesNotExist),
        OldStateRootDoesNotExist(OldStateRootDoesNotExist),
        OnlyAdmin(OnlyAdmin),
        OnlyPendingAdmin(OnlyPendingAdmin),
        OnlyRollupManager(OnlyRollupManager),
        OnlyTrustedAggregator(OnlyTrustedAggregator),
        OnlyTrustedSequencer(OnlyTrustedSequencer),
        PendingStateDoesNotExist(PendingStateDoesNotExist),
        PendingStateInvalid(PendingStateInvalid),
        PendingStateNotConsolidable(PendingStateNotConsolidable),
        PendingStateTimeoutExceedHaltAggregationTimeout(
            PendingStateTimeoutExceedHaltAggregationTimeout,
        ),
        SequenceZeroBatches(SequenceZeroBatches),
        SequencedTimestampBelowForcedTimestamp(SequencedTimestampBelowForcedTimestamp),
        SequencedTimestampInvalid(SequencedTimestampInvalid),
        StoredRootMustBeDifferentThanNewRoot(StoredRootMustBeDifferentThanNewRoot),
        TransactionsLengthAboveMax(TransactionsLengthAboveMax),
        TrustedAggregatorTimeoutExceedHaltAggregationTimeout(
            TrustedAggregatorTimeoutExceedHaltAggregationTimeout,
        ),
        TrustedAggregatorTimeoutNotExpired(TrustedAggregatorTimeoutNotExpired),
        /// The standard solidity revert string, with selector
        /// Error(string) -- 0x08c379a0
        RevertString(::std::string::String),
    }
    impl ::ethers::core::abi::AbiDecode for PolygonZkEvmErrors {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded) = <::std::string::String as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::RevertString(decoded));
            }
            if let Ok(decoded) = <BatchAlreadyVerified as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::BatchAlreadyVerified(decoded));
            }
            if let Ok(decoded) = <BatchNotSequencedOrNotSequenceEnd as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::BatchNotSequencedOrNotSequenceEnd(decoded));
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
            if let Ok(decoded) = <ForceBatchNotAllowed as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::ForceBatchNotAllowed(decoded));
            }
            if let Ok(decoded) = <ForceBatchTimeoutNotExpired as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::ForceBatchTimeoutNotExpired(decoded));
            }
            if let Ok(decoded) = <ForceBatchesAlreadyActive as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::ForceBatchesAlreadyActive(decoded));
            }
            if let Ok(decoded) = <ForceBatchesDecentralized as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::ForceBatchesDecentralized(decoded));
            }
            if let Ok(decoded) = <ForceBatchesNotAllowedOnEmergencyState as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::ForceBatchesNotAllowedOnEmergencyState(decoded));
            }
            if let Ok(decoded) = <ForceBatchesOverflow as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::ForceBatchesOverflow(decoded));
            }
            if let Ok(decoded) = <ForcedDataDoesNotMatch as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::ForcedDataDoesNotMatch(decoded));
            }
            if let Ok(decoded) = <GasTokenNetworkMustBeZeroOnEther as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GasTokenNetworkMustBeZeroOnEther(decoded));
            }
            if let Ok(decoded) = <GlobalExitRootNotExist as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GlobalExitRootNotExist(decoded));
            }
            if let Ok(decoded) = <HaltTimeoutNotExpired as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::HaltTimeoutNotExpired(decoded));
            }
            if let Ok(decoded) = <HaltTimeoutNotExpiredAfterEmergencyState as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::HaltTimeoutNotExpiredAfterEmergencyState(decoded));
            }
            if let Ok(decoded) = <HugeTokenMetadataNotSupported as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::HugeTokenMetadataNotSupported(decoded));
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
            if let Ok(decoded) = <InitSequencedBatchDoesNotMatch as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::InitSequencedBatchDoesNotMatch(decoded));
            }
            if let Ok(decoded) = <InvalidInitializeTransaction as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::InvalidInitializeTransaction(decoded));
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
            if let Ok(decoded) = <InvalidRangeForceBatchTimeout as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::InvalidRangeForceBatchTimeout(decoded));
            }
            if let Ok(decoded) = <InvalidRangeMultiplierBatchFee as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::InvalidRangeMultiplierBatchFee(decoded));
            }
            if let Ok(decoded) = <MaxTimestampSequenceInvalid as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::MaxTimestampSequenceInvalid(decoded));
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
            if let Ok(decoded) = <NotEnoughMaticAmount as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::NotEnoughMaticAmount(decoded));
            }
            if let Ok(decoded) = <NotEnoughPOLAmount as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::NotEnoughPOLAmount(decoded));
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
            if let Ok(decoded) = <OnlyAdmin as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::OnlyAdmin(decoded));
            }
            if let Ok(decoded) = <OnlyPendingAdmin as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::OnlyPendingAdmin(decoded));
            }
            if let Ok(decoded) = <OnlyRollupManager as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::OnlyRollupManager(decoded));
            }
            if let Ok(decoded) = <OnlyTrustedAggregator as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::OnlyTrustedAggregator(decoded));
            }
            if let Ok(decoded) = <OnlyTrustedSequencer as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::OnlyTrustedSequencer(decoded));
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
            if let Ok(decoded) = <PendingStateTimeoutExceedHaltAggregationTimeout as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(
                    Self::PendingStateTimeoutExceedHaltAggregationTimeout(decoded),
                );
            }
            if let Ok(decoded) = <SequenceZeroBatches as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::SequenceZeroBatches(decoded));
            }
            if let Ok(decoded) = <SequencedTimestampBelowForcedTimestamp as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::SequencedTimestampBelowForcedTimestamp(decoded));
            }
            if let Ok(decoded) = <SequencedTimestampInvalid as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::SequencedTimestampInvalid(decoded));
            }
            if let Ok(decoded) = <StoredRootMustBeDifferentThanNewRoot as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::StoredRootMustBeDifferentThanNewRoot(decoded));
            }
            if let Ok(decoded) = <TransactionsLengthAboveMax as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::TransactionsLengthAboveMax(decoded));
            }
            if let Ok(decoded) = <TrustedAggregatorTimeoutExceedHaltAggregationTimeout as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(
                    Self::TrustedAggregatorTimeoutExceedHaltAggregationTimeout(decoded),
                );
            }
            if let Ok(decoded) = <TrustedAggregatorTimeoutNotExpired as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::TrustedAggregatorTimeoutNotExpired(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for PolygonZkEvmErrors {
        fn encode(self) -> ::std::vec::Vec<u8> {
            match self {
                Self::BatchAlreadyVerified(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::BatchNotSequencedOrNotSequenceEnd(element) => {
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
                Self::ForceBatchNotAllowed(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ForceBatchTimeoutNotExpired(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ForceBatchesAlreadyActive(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ForceBatchesDecentralized(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ForceBatchesNotAllowedOnEmergencyState(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ForceBatchesOverflow(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ForcedDataDoesNotMatch(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GasTokenNetworkMustBeZeroOnEther(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GlobalExitRootNotExist(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::HaltTimeoutNotExpired(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::HaltTimeoutNotExpiredAfterEmergencyState(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::HugeTokenMetadataNotSupported(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InitNumBatchAboveLastVerifiedBatch(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InitNumBatchDoesNotMatchPendingState(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InitSequencedBatchDoesNotMatch(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidInitializeTransaction(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidProof(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidRangeBatchTimeTarget(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidRangeForceBatchTimeout(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidRangeMultiplierBatchFee(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::MaxTimestampSequenceInvalid(element) => {
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
                Self::NotEnoughMaticAmount(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NotEnoughPOLAmount(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::OldAccInputHashDoesNotExist(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::OldStateRootDoesNotExist(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::OnlyAdmin(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::OnlyPendingAdmin(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::OnlyRollupManager(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::OnlyTrustedAggregator(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::OnlyTrustedSequencer(element) => {
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
                Self::PendingStateTimeoutExceedHaltAggregationTimeout(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::SequenceZeroBatches(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::SequencedTimestampBelowForcedTimestamp(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::SequencedTimestampInvalid(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::StoredRootMustBeDifferentThanNewRoot(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::TransactionsLengthAboveMax(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::TrustedAggregatorTimeoutExceedHaltAggregationTimeout(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::TrustedAggregatorTimeoutNotExpired(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RevertString(s) => ::ethers::core::abi::AbiEncode::encode(s),
            }
        }
    }
    impl ::ethers::contract::ContractRevert for PolygonZkEvmErrors {
        fn valid_selector(selector: [u8; 4]) -> bool {
            match selector {
                [0x08, 0xc3, 0x79, 0xa0] => true,
                _ if selector
                    == <BatchAlreadyVerified as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <BatchNotSequencedOrNotSequenceEnd as ::ethers::contract::EthError>::selector() => {
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
                    == <ForceBatchNotAllowed as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <ForceBatchTimeoutNotExpired as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <ForceBatchesAlreadyActive as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <ForceBatchesDecentralized as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <ForceBatchesNotAllowedOnEmergencyState as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <ForceBatchesOverflow as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <ForcedDataDoesNotMatch as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <GasTokenNetworkMustBeZeroOnEther as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <GlobalExitRootNotExist as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <HaltTimeoutNotExpired as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <HaltTimeoutNotExpiredAfterEmergencyState as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <HugeTokenMetadataNotSupported as ::ethers::contract::EthError>::selector() => {
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
                    == <InitSequencedBatchDoesNotMatch as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InvalidInitializeTransaction as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InvalidProof as ::ethers::contract::EthError>::selector() => true,
                _ if selector
                    == <InvalidRangeBatchTimeTarget as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InvalidRangeForceBatchTimeout as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InvalidRangeMultiplierBatchFee as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <MaxTimestampSequenceInvalid as ::ethers::contract::EthError>::selector() => {
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
                    == <NotEnoughMaticAmount as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <NotEnoughPOLAmount as ::ethers::contract::EthError>::selector() => {
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
                    == <OnlyAdmin as ::ethers::contract::EthError>::selector() => true,
                _ if selector
                    == <OnlyPendingAdmin as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <OnlyRollupManager as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <OnlyTrustedAggregator as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <OnlyTrustedSequencer as ::ethers::contract::EthError>::selector() => {
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
                    == <PendingStateTimeoutExceedHaltAggregationTimeout as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <SequenceZeroBatches as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <SequencedTimestampBelowForcedTimestamp as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <SequencedTimestampInvalid as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <StoredRootMustBeDifferentThanNewRoot as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <TransactionsLengthAboveMax as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <TrustedAggregatorTimeoutExceedHaltAggregationTimeout as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <TrustedAggregatorTimeoutNotExpired as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ => false,
            }
        }
    }
    impl ::core::fmt::Display for PolygonZkEvmErrors {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::BatchAlreadyVerified(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::BatchNotSequencedOrNotSequenceEnd(element) => {
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
                Self::ForceBatchNotAllowed(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ForceBatchTimeoutNotExpired(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ForceBatchesAlreadyActive(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ForceBatchesDecentralized(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ForceBatchesNotAllowedOnEmergencyState(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ForceBatchesOverflow(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ForcedDataDoesNotMatch(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::GasTokenNetworkMustBeZeroOnEther(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::GlobalExitRootNotExist(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::HaltTimeoutNotExpired(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::HaltTimeoutNotExpiredAfterEmergencyState(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::HugeTokenMetadataNotSupported(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InitNumBatchAboveLastVerifiedBatch(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InitNumBatchDoesNotMatchPendingState(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InitSequencedBatchDoesNotMatch(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InvalidInitializeTransaction(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InvalidProof(element) => ::core::fmt::Display::fmt(element, f),
                Self::InvalidRangeBatchTimeTarget(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InvalidRangeForceBatchTimeout(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InvalidRangeMultiplierBatchFee(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::MaxTimestampSequenceInvalid(element) => {
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
                Self::NotEnoughMaticAmount(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::NotEnoughPOLAmount(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::OldAccInputHashDoesNotExist(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::OldStateRootDoesNotExist(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::OnlyAdmin(element) => ::core::fmt::Display::fmt(element, f),
                Self::OnlyPendingAdmin(element) => ::core::fmt::Display::fmt(element, f),
                Self::OnlyRollupManager(element) => ::core::fmt::Display::fmt(element, f),
                Self::OnlyTrustedAggregator(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::OnlyTrustedSequencer(element) => {
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
                Self::PendingStateTimeoutExceedHaltAggregationTimeout(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::SequenceZeroBatches(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::SequencedTimestampBelowForcedTimestamp(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::SequencedTimestampInvalid(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::StoredRootMustBeDifferentThanNewRoot(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::TransactionsLengthAboveMax(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::TrustedAggregatorTimeoutExceedHaltAggregationTimeout(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::TrustedAggregatorTimeoutNotExpired(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RevertString(s) => ::core::fmt::Display::fmt(s, f),
            }
        }
    }
    impl ::core::convert::From<::std::string::String> for PolygonZkEvmErrors {
        fn from(value: String) -> Self {
            Self::RevertString(value)
        }
    }
    impl ::core::convert::From<BatchAlreadyVerified> for PolygonZkEvmErrors {
        fn from(value: BatchAlreadyVerified) -> Self {
            Self::BatchAlreadyVerified(value)
        }
    }
    impl ::core::convert::From<BatchNotSequencedOrNotSequenceEnd>
    for PolygonZkEvmErrors {
        fn from(value: BatchNotSequencedOrNotSequenceEnd) -> Self {
            Self::BatchNotSequencedOrNotSequenceEnd(value)
        }
    }
    impl ::core::convert::From<ExceedMaxVerifyBatches> for PolygonZkEvmErrors {
        fn from(value: ExceedMaxVerifyBatches) -> Self {
            Self::ExceedMaxVerifyBatches(value)
        }
    }
    impl ::core::convert::From<FinalNumBatchBelowLastVerifiedBatch>
    for PolygonZkEvmErrors {
        fn from(value: FinalNumBatchBelowLastVerifiedBatch) -> Self {
            Self::FinalNumBatchBelowLastVerifiedBatch(value)
        }
    }
    impl ::core::convert::From<FinalNumBatchDoesNotMatchPendingState>
    for PolygonZkEvmErrors {
        fn from(value: FinalNumBatchDoesNotMatchPendingState) -> Self {
            Self::FinalNumBatchDoesNotMatchPendingState(value)
        }
    }
    impl ::core::convert::From<FinalPendingStateNumInvalid> for PolygonZkEvmErrors {
        fn from(value: FinalPendingStateNumInvalid) -> Self {
            Self::FinalPendingStateNumInvalid(value)
        }
    }
    impl ::core::convert::From<ForceBatchNotAllowed> for PolygonZkEvmErrors {
        fn from(value: ForceBatchNotAllowed) -> Self {
            Self::ForceBatchNotAllowed(value)
        }
    }
    impl ::core::convert::From<ForceBatchTimeoutNotExpired> for PolygonZkEvmErrors {
        fn from(value: ForceBatchTimeoutNotExpired) -> Self {
            Self::ForceBatchTimeoutNotExpired(value)
        }
    }
    impl ::core::convert::From<ForceBatchesAlreadyActive> for PolygonZkEvmErrors {
        fn from(value: ForceBatchesAlreadyActive) -> Self {
            Self::ForceBatchesAlreadyActive(value)
        }
    }
    impl ::core::convert::From<ForceBatchesDecentralized> for PolygonZkEvmErrors {
        fn from(value: ForceBatchesDecentralized) -> Self {
            Self::ForceBatchesDecentralized(value)
        }
    }
    impl ::core::convert::From<ForceBatchesNotAllowedOnEmergencyState>
    for PolygonZkEvmErrors {
        fn from(value: ForceBatchesNotAllowedOnEmergencyState) -> Self {
            Self::ForceBatchesNotAllowedOnEmergencyState(value)
        }
    }
    impl ::core::convert::From<ForceBatchesOverflow> for PolygonZkEvmErrors {
        fn from(value: ForceBatchesOverflow) -> Self {
            Self::ForceBatchesOverflow(value)
        }
    }
    impl ::core::convert::From<ForcedDataDoesNotMatch> for PolygonZkEvmErrors {
        fn from(value: ForcedDataDoesNotMatch) -> Self {
            Self::ForcedDataDoesNotMatch(value)
        }
    }
    impl ::core::convert::From<GasTokenNetworkMustBeZeroOnEther> for PolygonZkEvmErrors {
        fn from(value: GasTokenNetworkMustBeZeroOnEther) -> Self {
            Self::GasTokenNetworkMustBeZeroOnEther(value)
        }
    }
    impl ::core::convert::From<GlobalExitRootNotExist> for PolygonZkEvmErrors {
        fn from(value: GlobalExitRootNotExist) -> Self {
            Self::GlobalExitRootNotExist(value)
        }
    }
    impl ::core::convert::From<HaltTimeoutNotExpired> for PolygonZkEvmErrors {
        fn from(value: HaltTimeoutNotExpired) -> Self {
            Self::HaltTimeoutNotExpired(value)
        }
    }
    impl ::core::convert::From<HaltTimeoutNotExpiredAfterEmergencyState>
    for PolygonZkEvmErrors {
        fn from(value: HaltTimeoutNotExpiredAfterEmergencyState) -> Self {
            Self::HaltTimeoutNotExpiredAfterEmergencyState(value)
        }
    }
    impl ::core::convert::From<HugeTokenMetadataNotSupported> for PolygonZkEvmErrors {
        fn from(value: HugeTokenMetadataNotSupported) -> Self {
            Self::HugeTokenMetadataNotSupported(value)
        }
    }
    impl ::core::convert::From<InitNumBatchAboveLastVerifiedBatch>
    for PolygonZkEvmErrors {
        fn from(value: InitNumBatchAboveLastVerifiedBatch) -> Self {
            Self::InitNumBatchAboveLastVerifiedBatch(value)
        }
    }
    impl ::core::convert::From<InitNumBatchDoesNotMatchPendingState>
    for PolygonZkEvmErrors {
        fn from(value: InitNumBatchDoesNotMatchPendingState) -> Self {
            Self::InitNumBatchDoesNotMatchPendingState(value)
        }
    }
    impl ::core::convert::From<InitSequencedBatchDoesNotMatch> for PolygonZkEvmErrors {
        fn from(value: InitSequencedBatchDoesNotMatch) -> Self {
            Self::InitSequencedBatchDoesNotMatch(value)
        }
    }
    impl ::core::convert::From<InvalidInitializeTransaction> for PolygonZkEvmErrors {
        fn from(value: InvalidInitializeTransaction) -> Self {
            Self::InvalidInitializeTransaction(value)
        }
    }
    impl ::core::convert::From<InvalidProof> for PolygonZkEvmErrors {
        fn from(value: InvalidProof) -> Self {
            Self::InvalidProof(value)
        }
    }
    impl ::core::convert::From<InvalidRangeBatchTimeTarget> for PolygonZkEvmErrors {
        fn from(value: InvalidRangeBatchTimeTarget) -> Self {
            Self::InvalidRangeBatchTimeTarget(value)
        }
    }
    impl ::core::convert::From<InvalidRangeForceBatchTimeout> for PolygonZkEvmErrors {
        fn from(value: InvalidRangeForceBatchTimeout) -> Self {
            Self::InvalidRangeForceBatchTimeout(value)
        }
    }
    impl ::core::convert::From<InvalidRangeMultiplierBatchFee> for PolygonZkEvmErrors {
        fn from(value: InvalidRangeMultiplierBatchFee) -> Self {
            Self::InvalidRangeMultiplierBatchFee(value)
        }
    }
    impl ::core::convert::From<MaxTimestampSequenceInvalid> for PolygonZkEvmErrors {
        fn from(value: MaxTimestampSequenceInvalid) -> Self {
            Self::MaxTimestampSequenceInvalid(value)
        }
    }
    impl ::core::convert::From<NewAccInputHashDoesNotExist> for PolygonZkEvmErrors {
        fn from(value: NewAccInputHashDoesNotExist) -> Self {
            Self::NewAccInputHashDoesNotExist(value)
        }
    }
    impl ::core::convert::From<NewPendingStateTimeoutMustBeLower>
    for PolygonZkEvmErrors {
        fn from(value: NewPendingStateTimeoutMustBeLower) -> Self {
            Self::NewPendingStateTimeoutMustBeLower(value)
        }
    }
    impl ::core::convert::From<NewStateRootNotInsidePrime> for PolygonZkEvmErrors {
        fn from(value: NewStateRootNotInsidePrime) -> Self {
            Self::NewStateRootNotInsidePrime(value)
        }
    }
    impl ::core::convert::From<NewTrustedAggregatorTimeoutMustBeLower>
    for PolygonZkEvmErrors {
        fn from(value: NewTrustedAggregatorTimeoutMustBeLower) -> Self {
            Self::NewTrustedAggregatorTimeoutMustBeLower(value)
        }
    }
    impl ::core::convert::From<NotEnoughMaticAmount> for PolygonZkEvmErrors {
        fn from(value: NotEnoughMaticAmount) -> Self {
            Self::NotEnoughMaticAmount(value)
        }
    }
    impl ::core::convert::From<NotEnoughPOLAmount> for PolygonZkEvmErrors {
        fn from(value: NotEnoughPOLAmount) -> Self {
            Self::NotEnoughPOLAmount(value)
        }
    }
    impl ::core::convert::From<OldAccInputHashDoesNotExist> for PolygonZkEvmErrors {
        fn from(value: OldAccInputHashDoesNotExist) -> Self {
            Self::OldAccInputHashDoesNotExist(value)
        }
    }
    impl ::core::convert::From<OldStateRootDoesNotExist> for PolygonZkEvmErrors {
        fn from(value: OldStateRootDoesNotExist) -> Self {
            Self::OldStateRootDoesNotExist(value)
        }
    }
    impl ::core::convert::From<OnlyAdmin> for PolygonZkEvmErrors {
        fn from(value: OnlyAdmin) -> Self {
            Self::OnlyAdmin(value)
        }
    }
    impl ::core::convert::From<OnlyPendingAdmin> for PolygonZkEvmErrors {
        fn from(value: OnlyPendingAdmin) -> Self {
            Self::OnlyPendingAdmin(value)
        }
    }
    impl ::core::convert::From<OnlyRollupManager> for PolygonZkEvmErrors {
        fn from(value: OnlyRollupManager) -> Self {
            Self::OnlyRollupManager(value)
        }
    }
    impl ::core::convert::From<OnlyTrustedAggregator> for PolygonZkEvmErrors {
        fn from(value: OnlyTrustedAggregator) -> Self {
            Self::OnlyTrustedAggregator(value)
        }
    }
    impl ::core::convert::From<OnlyTrustedSequencer> for PolygonZkEvmErrors {
        fn from(value: OnlyTrustedSequencer) -> Self {
            Self::OnlyTrustedSequencer(value)
        }
    }
    impl ::core::convert::From<PendingStateDoesNotExist> for PolygonZkEvmErrors {
        fn from(value: PendingStateDoesNotExist) -> Self {
            Self::PendingStateDoesNotExist(value)
        }
    }
    impl ::core::convert::From<PendingStateInvalid> for PolygonZkEvmErrors {
        fn from(value: PendingStateInvalid) -> Self {
            Self::PendingStateInvalid(value)
        }
    }
    impl ::core::convert::From<PendingStateNotConsolidable> for PolygonZkEvmErrors {
        fn from(value: PendingStateNotConsolidable) -> Self {
            Self::PendingStateNotConsolidable(value)
        }
    }
    impl ::core::convert::From<PendingStateTimeoutExceedHaltAggregationTimeout>
    for PolygonZkEvmErrors {
        fn from(value: PendingStateTimeoutExceedHaltAggregationTimeout) -> Self {
            Self::PendingStateTimeoutExceedHaltAggregationTimeout(value)
        }
    }
    impl ::core::convert::From<SequenceZeroBatches> for PolygonZkEvmErrors {
        fn from(value: SequenceZeroBatches) -> Self {
            Self::SequenceZeroBatches(value)
        }
    }
    impl ::core::convert::From<SequencedTimestampBelowForcedTimestamp>
    for PolygonZkEvmErrors {
        fn from(value: SequencedTimestampBelowForcedTimestamp) -> Self {
            Self::SequencedTimestampBelowForcedTimestamp(value)
        }
    }
    impl ::core::convert::From<SequencedTimestampInvalid> for PolygonZkEvmErrors {
        fn from(value: SequencedTimestampInvalid) -> Self {
            Self::SequencedTimestampInvalid(value)
        }
    }
    impl ::core::convert::From<StoredRootMustBeDifferentThanNewRoot>
    for PolygonZkEvmErrors {
        fn from(value: StoredRootMustBeDifferentThanNewRoot) -> Self {
            Self::StoredRootMustBeDifferentThanNewRoot(value)
        }
    }
    impl ::core::convert::From<TransactionsLengthAboveMax> for PolygonZkEvmErrors {
        fn from(value: TransactionsLengthAboveMax) -> Self {
            Self::TransactionsLengthAboveMax(value)
        }
    }
    impl ::core::convert::From<TrustedAggregatorTimeoutExceedHaltAggregationTimeout>
    for PolygonZkEvmErrors {
        fn from(value: TrustedAggregatorTimeoutExceedHaltAggregationTimeout) -> Self {
            Self::TrustedAggregatorTimeoutExceedHaltAggregationTimeout(value)
        }
    }
    impl ::core::convert::From<TrustedAggregatorTimeoutNotExpired>
    for PolygonZkEvmErrors {
        fn from(value: TrustedAggregatorTimeoutNotExpired) -> Self {
            Self::TrustedAggregatorTimeoutNotExpired(value)
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
    #[ethevent(name = "AcceptAdminRole", abi = "AcceptAdminRole(address)")]
    pub struct AcceptAdminRoleFilter {
        pub new_admin: ::ethers::core::types::Address,
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
    #[ethevent(name = "ForceBatch", abi = "ForceBatch(uint64,bytes32,address,bytes)")]
    pub struct ForceBatchFilter {
        #[ethevent(indexed)]
        pub force_batch_num: u64,
        pub last_global_exit_root: [u8; 32],
        pub sequencer: ::ethers::core::types::Address,
        pub transactions: ::ethers::core::types::Bytes,
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
        name = "InitialSequenceBatches",
        abi = "InitialSequenceBatches(bytes,bytes32,address)"
    )]
    pub struct InitialSequenceBatchesFilter {
        pub transactions: ::ethers::core::types::Bytes,
        pub last_global_exit_root: [u8; 32],
        pub sequencer: ::ethers::core::types::Address,
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
    #[ethevent(name = "SequenceBatches", abi = "SequenceBatches(uint64,bytes32)")]
    pub struct SequenceBatchesFilter {
        #[ethevent(indexed)]
        pub num_batch: u64,
        pub l_1_info_root: [u8; 32],
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
    #[ethevent(name = "SequenceForceBatches", abi = "SequenceForceBatches(uint64)")]
    pub struct SequenceForceBatchesFilter {
        #[ethevent(indexed)]
        pub num_batch: u64,
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
    #[ethevent(name = "SetForceBatchAddress", abi = "SetForceBatchAddress(address)")]
    pub struct SetForceBatchAddressFilter {
        pub new_force_batch_address: ::ethers::core::types::Address,
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
    #[ethevent(name = "SetForceBatchTimeout", abi = "SetForceBatchTimeout(uint64)")]
    pub struct SetForceBatchTimeoutFilter {
        pub newforce_batch_timeout: u64,
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
    #[ethevent(name = "SetTrustedSequencer", abi = "SetTrustedSequencer(address)")]
    pub struct SetTrustedSequencerFilter {
        pub new_trusted_sequencer: ::ethers::core::types::Address,
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
    #[ethevent(name = "SetTrustedSequencerURL", abi = "SetTrustedSequencerURL(string)")]
    pub struct SetTrustedSequencerURLFilter {
        pub new_trusted_sequencer_url: ::std::string::String,
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
    #[ethevent(name = "TransferAdminRole", abi = "TransferAdminRole(address)")]
    pub struct TransferAdminRoleFilter {
        pub new_pending_admin: ::ethers::core::types::Address,
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
    #[ethevent(name = "VerifyBatches", abi = "VerifyBatches(uint64,bytes32,address)")]
    pub struct VerifyBatchesFilter {
        #[ethevent(indexed)]
        pub num_batch: u64,
        pub state_root: [u8; 32],
        #[ethevent(indexed)]
        pub aggregator: ::ethers::core::types::Address,
    }
    ///Container type for all of the contract's events
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum PolygonZkEvmEvents {
        AcceptAdminRoleFilter(AcceptAdminRoleFilter),
        ForceBatchFilter(ForceBatchFilter),
        InitialSequenceBatchesFilter(InitialSequenceBatchesFilter),
        InitializedFilter(InitializedFilter),
        SequenceBatchesFilter(SequenceBatchesFilter),
        SequenceForceBatchesFilter(SequenceForceBatchesFilter),
        SetForceBatchAddressFilter(SetForceBatchAddressFilter),
        SetForceBatchTimeoutFilter(SetForceBatchTimeoutFilter),
        SetTrustedSequencerFilter(SetTrustedSequencerFilter),
        SetTrustedSequencerURLFilter(SetTrustedSequencerURLFilter),
        TransferAdminRoleFilter(TransferAdminRoleFilter),
        VerifyBatchesFilter(VerifyBatchesFilter),
    }
    impl ::ethers::contract::EthLogDecode for PolygonZkEvmEvents {
        fn decode_log(
            log: &::ethers::core::abi::RawLog,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::Error> {
            if let Ok(decoded) = AcceptAdminRoleFilter::decode_log(log) {
                return Ok(PolygonZkEvmEvents::AcceptAdminRoleFilter(decoded));
            }
            if let Ok(decoded) = ForceBatchFilter::decode_log(log) {
                return Ok(PolygonZkEvmEvents::ForceBatchFilter(decoded));
            }
            if let Ok(decoded) = InitialSequenceBatchesFilter::decode_log(log) {
                return Ok(PolygonZkEvmEvents::InitialSequenceBatchesFilter(decoded));
            }
            if let Ok(decoded) = InitializedFilter::decode_log(log) {
                return Ok(PolygonZkEvmEvents::InitializedFilter(decoded));
            }
            if let Ok(decoded) = SequenceBatchesFilter::decode_log(log) {
                return Ok(PolygonZkEvmEvents::SequenceBatchesFilter(decoded));
            }
            if let Ok(decoded) = SequenceForceBatchesFilter::decode_log(log) {
                return Ok(PolygonZkEvmEvents::SequenceForceBatchesFilter(decoded));
            }
            if let Ok(decoded) = SetForceBatchAddressFilter::decode_log(log) {
                return Ok(PolygonZkEvmEvents::SetForceBatchAddressFilter(decoded));
            }
            if let Ok(decoded) = SetForceBatchTimeoutFilter::decode_log(log) {
                return Ok(PolygonZkEvmEvents::SetForceBatchTimeoutFilter(decoded));
            }
            if let Ok(decoded) = SetTrustedSequencerFilter::decode_log(log) {
                return Ok(PolygonZkEvmEvents::SetTrustedSequencerFilter(decoded));
            }
            if let Ok(decoded) = SetTrustedSequencerURLFilter::decode_log(log) {
                return Ok(PolygonZkEvmEvents::SetTrustedSequencerURLFilter(decoded));
            }
            if let Ok(decoded) = TransferAdminRoleFilter::decode_log(log) {
                return Ok(PolygonZkEvmEvents::TransferAdminRoleFilter(decoded));
            }
            if let Ok(decoded) = VerifyBatchesFilter::decode_log(log) {
                return Ok(PolygonZkEvmEvents::VerifyBatchesFilter(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::core::fmt::Display for PolygonZkEvmEvents {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::AcceptAdminRoleFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ForceBatchFilter(element) => ::core::fmt::Display::fmt(element, f),
                Self::InitialSequenceBatchesFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InitializedFilter(element) => ::core::fmt::Display::fmt(element, f),
                Self::SequenceBatchesFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::SequenceForceBatchesFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::SetForceBatchAddressFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::SetForceBatchTimeoutFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::SetTrustedSequencerFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::SetTrustedSequencerURLFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::TransferAdminRoleFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::VerifyBatchesFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
            }
        }
    }
    impl ::core::convert::From<AcceptAdminRoleFilter> for PolygonZkEvmEvents {
        fn from(value: AcceptAdminRoleFilter) -> Self {
            Self::AcceptAdminRoleFilter(value)
        }
    }
    impl ::core::convert::From<ForceBatchFilter> for PolygonZkEvmEvents {
        fn from(value: ForceBatchFilter) -> Self {
            Self::ForceBatchFilter(value)
        }
    }
    impl ::core::convert::From<InitialSequenceBatchesFilter> for PolygonZkEvmEvents {
        fn from(value: InitialSequenceBatchesFilter) -> Self {
            Self::InitialSequenceBatchesFilter(value)
        }
    }
    impl ::core::convert::From<InitializedFilter> for PolygonZkEvmEvents {
        fn from(value: InitializedFilter) -> Self {
            Self::InitializedFilter(value)
        }
    }
    impl ::core::convert::From<SequenceBatchesFilter> for PolygonZkEvmEvents {
        fn from(value: SequenceBatchesFilter) -> Self {
            Self::SequenceBatchesFilter(value)
        }
    }
    impl ::core::convert::From<SequenceForceBatchesFilter> for PolygonZkEvmEvents {
        fn from(value: SequenceForceBatchesFilter) -> Self {
            Self::SequenceForceBatchesFilter(value)
        }
    }
    impl ::core::convert::From<SetForceBatchAddressFilter> for PolygonZkEvmEvents {
        fn from(value: SetForceBatchAddressFilter) -> Self {
            Self::SetForceBatchAddressFilter(value)
        }
    }
    impl ::core::convert::From<SetForceBatchTimeoutFilter> for PolygonZkEvmEvents {
        fn from(value: SetForceBatchTimeoutFilter) -> Self {
            Self::SetForceBatchTimeoutFilter(value)
        }
    }
    impl ::core::convert::From<SetTrustedSequencerFilter> for PolygonZkEvmEvents {
        fn from(value: SetTrustedSequencerFilter) -> Self {
            Self::SetTrustedSequencerFilter(value)
        }
    }
    impl ::core::convert::From<SetTrustedSequencerURLFilter> for PolygonZkEvmEvents {
        fn from(value: SetTrustedSequencerURLFilter) -> Self {
            Self::SetTrustedSequencerURLFilter(value)
        }
    }
    impl ::core::convert::From<TransferAdminRoleFilter> for PolygonZkEvmEvents {
        fn from(value: TransferAdminRoleFilter) -> Self {
            Self::TransferAdminRoleFilter(value)
        }
    }
    impl ::core::convert::From<VerifyBatchesFilter> for PolygonZkEvmEvents {
        fn from(value: VerifyBatchesFilter) -> Self {
            Self::VerifyBatchesFilter(value)
        }
    }
    ///Container type for all input parameters for the `GLOBAL_EXIT_ROOT_MANAGER_L2` function with signature `GLOBAL_EXIT_ROOT_MANAGER_L2()` and selector `0x9e001877`
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
        name = "GLOBAL_EXIT_ROOT_MANAGER_L2",
        abi = "GLOBAL_EXIT_ROOT_MANAGER_L2()"
    )]
    pub struct GlobalExitRootManagerL2Call;
    ///Container type for all input parameters for the `INITIALIZE_TX_BRIDGE_LIST_LEN_LEN` function with signature `INITIALIZE_TX_BRIDGE_LIST_LEN_LEN()` and selector `0x11e892d4`
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
        name = "INITIALIZE_TX_BRIDGE_LIST_LEN_LEN",
        abi = "INITIALIZE_TX_BRIDGE_LIST_LEN_LEN()"
    )]
    pub struct InitializeTxBridgeListLenLenCall;
    ///Container type for all input parameters for the `INITIALIZE_TX_BRIDGE_PARAMS` function with signature `INITIALIZE_TX_BRIDGE_PARAMS()` and selector `0x05835f37`
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
        name = "INITIALIZE_TX_BRIDGE_PARAMS",
        abi = "INITIALIZE_TX_BRIDGE_PARAMS()"
    )]
    pub struct InitializeTxBridgeParamsCall;
    ///Container type for all input parameters for the `INITIALIZE_TX_BRIDGE_PARAMS_AFTER_BRIDGE_ADDRESS` function with signature `INITIALIZE_TX_BRIDGE_PARAMS_AFTER_BRIDGE_ADDRESS()` and selector `0x7a5460c5`
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
        name = "INITIALIZE_TX_BRIDGE_PARAMS_AFTER_BRIDGE_ADDRESS",
        abi = "INITIALIZE_TX_BRIDGE_PARAMS_AFTER_BRIDGE_ADDRESS()"
    )]
    pub struct InitializeTxBridgeParamsAfterBridgeAddressCall;
    ///Container type for all input parameters for the `INITIALIZE_TX_BRIDGE_PARAMS_AFTER_BRIDGE_ADDRESS_EMPTY_METADATA` function with signature `INITIALIZE_TX_BRIDGE_PARAMS_AFTER_BRIDGE_ADDRESS_EMPTY_METADATA()` and selector `0x52bdeb6d`
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
        name = "INITIALIZE_TX_BRIDGE_PARAMS_AFTER_BRIDGE_ADDRESS_EMPTY_METADATA",
        abi = "INITIALIZE_TX_BRIDGE_PARAMS_AFTER_BRIDGE_ADDRESS_EMPTY_METADATA()"
    )]
    pub struct InitializeTxBridgeParamsAfterBridgeAddressEmptyMetadataCall;
    ///Container type for all input parameters for the `INITIALIZE_TX_CONSTANT_BYTES` function with signature `INITIALIZE_TX_CONSTANT_BYTES()` and selector `0x03508963`
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
        name = "INITIALIZE_TX_CONSTANT_BYTES",
        abi = "INITIALIZE_TX_CONSTANT_BYTES()"
    )]
    pub struct InitializeTxConstantBytesCall;
    ///Container type for all input parameters for the `INITIALIZE_TX_CONSTANT_BYTES_EMPTY_METADATA` function with signature `INITIALIZE_TX_CONSTANT_BYTES_EMPTY_METADATA()` and selector `0x676870d2`
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
        name = "INITIALIZE_TX_CONSTANT_BYTES_EMPTY_METADATA",
        abi = "INITIALIZE_TX_CONSTANT_BYTES_EMPTY_METADATA()"
    )]
    pub struct InitializeTxConstantBytesEmptyMetadataCall;
    ///Container type for all input parameters for the `INITIALIZE_TX_DATA_LEN_EMPTY_METADATA` function with signature `INITIALIZE_TX_DATA_LEN_EMPTY_METADATA()` and selector `0xc7fffd4b`
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
        name = "INITIALIZE_TX_DATA_LEN_EMPTY_METADATA",
        abi = "INITIALIZE_TX_DATA_LEN_EMPTY_METADATA()"
    )]
    pub struct InitializeTxDataLenEmptyMetadataCall;
    ///Container type for all input parameters for the `INITIALIZE_TX_EFFECTIVE_PERCENTAGE` function with signature `INITIALIZE_TX_EFFECTIVE_PERCENTAGE()` and selector `0x40b5de6c`
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
        name = "INITIALIZE_TX_EFFECTIVE_PERCENTAGE",
        abi = "INITIALIZE_TX_EFFECTIVE_PERCENTAGE()"
    )]
    pub struct InitializeTxEffectivePercentageCall;
    ///Container type for all input parameters for the `SIGNATURE_INITIALIZE_TX_R` function with signature `SIGNATURE_INITIALIZE_TX_R()` and selector `0xb0afe154`
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
    #[ethcall(name = "SIGNATURE_INITIALIZE_TX_R", abi = "SIGNATURE_INITIALIZE_TX_R()")]
    pub struct SignatureInitializeTxRCall;
    ///Container type for all input parameters for the `SIGNATURE_INITIALIZE_TX_S` function with signature `SIGNATURE_INITIALIZE_TX_S()` and selector `0xd7bc90ff`
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
    #[ethcall(name = "SIGNATURE_INITIALIZE_TX_S", abi = "SIGNATURE_INITIALIZE_TX_S()")]
    pub struct SignatureInitializeTxSCall;
    ///Container type for all input parameters for the `SIGNATURE_INITIALIZE_TX_V` function with signature `SIGNATURE_INITIALIZE_TX_V()` and selector `0xf35dda47`
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
    #[ethcall(name = "SIGNATURE_INITIALIZE_TX_V", abi = "SIGNATURE_INITIALIZE_TX_V()")]
    pub struct SignatureInitializeTxVCall;
    ///Container type for all input parameters for the `TIMESTAMP_RANGE` function with signature `TIMESTAMP_RANGE()` and selector `0x42308fab`
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
    #[ethcall(name = "TIMESTAMP_RANGE", abi = "TIMESTAMP_RANGE()")]
    pub struct TimestampRangeCall;
    ///Container type for all input parameters for the `acceptAdminRole` function with signature `acceptAdminRole()` and selector `0x8c3d7301`
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
    #[ethcall(name = "acceptAdminRole", abi = "acceptAdminRole()")]
    pub struct AcceptAdminRoleCall;
    ///Container type for all input parameters for the `admin` function with signature `admin()` and selector `0xf851a440`
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
    #[ethcall(name = "admin", abi = "admin()")]
    pub struct AdminCall;
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
    ///Container type for all input parameters for the `calculatePolPerForceBatch` function with signature `calculatePolPerForceBatch()` and selector `0x00d0295d`
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
    #[ethcall(name = "calculatePolPerForceBatch", abi = "calculatePolPerForceBatch()")]
    pub struct CalculatePolPerForceBatchCall;
    ///Container type for all input parameters for the `forceBatch` function with signature `forceBatch(bytes,uint256)` and selector `0xeaeb077b`
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
    #[ethcall(name = "forceBatch", abi = "forceBatch(bytes,uint256)")]
    pub struct ForceBatchCall {
        pub transactions: ::ethers::core::types::Bytes,
        pub pol_amount: ::ethers::core::types::U256,
    }
    ///Container type for all input parameters for the `forceBatchAddress` function with signature `forceBatchAddress()` and selector `0x2c111c06`
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
    #[ethcall(name = "forceBatchAddress", abi = "forceBatchAddress()")]
    pub struct ForceBatchAddressCall;
    ///Container type for all input parameters for the `forceBatchTimeout` function with signature `forceBatchTimeout()` and selector `0xc754c7ed`
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
    #[ethcall(name = "forceBatchTimeout", abi = "forceBatchTimeout()")]
    pub struct ForceBatchTimeoutCall;
    ///Container type for all input parameters for the `forcedBatches` function with signature `forcedBatches(uint64)` and selector `0x6b8616ce`
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
    #[ethcall(name = "forcedBatches", abi = "forcedBatches(uint64)")]
    pub struct ForcedBatchesCall(pub u64);
    ///Container type for all input parameters for the `gasTokenAddress` function with signature `gasTokenAddress()` and selector `0x3c351e10`
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
    #[ethcall(name = "gasTokenAddress", abi = "gasTokenAddress()")]
    pub struct GasTokenAddressCall;
    ///Container type for all input parameters for the `gasTokenNetwork` function with signature `gasTokenNetwork()` and selector `0x3cbc795b`
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
    #[ethcall(name = "gasTokenNetwork", abi = "gasTokenNetwork()")]
    pub struct GasTokenNetworkCall;
    ///Container type for all input parameters for the `generateInitializeTransaction` function with signature `generateInitializeTransaction(uint32,address,uint32,bytes)` and selector `0xa652f26c`
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
        name = "generateInitializeTransaction",
        abi = "generateInitializeTransaction(uint32,address,uint32,bytes)"
    )]
    pub struct GenerateInitializeTransactionCall {
        pub network_id: u32,
        pub gas_token_address: ::ethers::core::types::Address,
        pub gas_token_network: u32,
        pub gas_token_metadata: ::ethers::core::types::Bytes,
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
    ///Container type for all input parameters for the `initialize` function with signature `initialize(address,address,uint32,address,string,string)` and selector `0x71257022`
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
        abi = "initialize(address,address,uint32,address,string,string)"
    )]
    pub struct InitializeCall {
        pub admin: ::ethers::core::types::Address,
        pub sequencer: ::ethers::core::types::Address,
        pub network_id: u32,
        pub gas_token_address: ::ethers::core::types::Address,
        pub sequencer_url: ::std::string::String,
        pub network_name: ::std::string::String,
    }
    ///Container type for all input parameters for the `lastAccInputHash` function with signature `lastAccInputHash()` and selector `0x6e05d2cd`
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
    #[ethcall(name = "lastAccInputHash", abi = "lastAccInputHash()")]
    pub struct LastAccInputHashCall;
    ///Container type for all input parameters for the `lastForceBatch` function with signature `lastForceBatch()` and selector `0xe7a7ed02`
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
    #[ethcall(name = "lastForceBatch", abi = "lastForceBatch()")]
    pub struct LastForceBatchCall;
    ///Container type for all input parameters for the `lastForceBatchSequenced` function with signature `lastForceBatchSequenced()` and selector `0x45605267`
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
    #[ethcall(name = "lastForceBatchSequenced", abi = "lastForceBatchSequenced()")]
    pub struct LastForceBatchSequencedCall;
    ///Container type for all input parameters for the `networkName` function with signature `networkName()` and selector `0x107bf28c`
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
    #[ethcall(name = "networkName", abi = "networkName()")]
    pub struct NetworkNameCall;
    ///Container type for all input parameters for the `onVerifyBatches` function with signature `onVerifyBatches(uint64,bytes32,address)` and selector `0x32c2d153`
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
    #[ethcall(name = "onVerifyBatches", abi = "onVerifyBatches(uint64,bytes32,address)")]
    pub struct OnVerifyBatchesCall {
        pub last_verified_batch: u64,
        pub new_state_root: [u8; 32],
        pub aggregator: ::ethers::core::types::Address,
    }
    ///Container type for all input parameters for the `pendingAdmin` function with signature `pendingAdmin()` and selector `0x26782247`
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
    #[ethcall(name = "pendingAdmin", abi = "pendingAdmin()")]
    pub struct PendingAdminCall;
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
    ///Container type for all input parameters for the `rollupManager` function with signature `rollupManager()` and selector `0x49b7b802`
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
    #[ethcall(name = "rollupManager", abi = "rollupManager()")]
    pub struct RollupManagerCall;
    ///Container type for all input parameters for the `sequenceBatches` function with signature `sequenceBatches((bytes,bytes32,uint64,bytes32)[],uint64,uint64,address)` and selector `0xdef57e54`
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
        name = "sequenceBatches",
        abi = "sequenceBatches((bytes,bytes32,uint64,bytes32)[],uint64,uint64,address)"
    )]
    pub struct SequenceBatchesCall {
        pub batches: ::std::vec::Vec<BatchData>,
        pub max_sequence_timestamp: u64,
        pub init_sequenced_batch: u64,
        pub l_2_coinbase: ::ethers::core::types::Address,
    }
    ///Container type for all input parameters for the `sequenceForceBatches` function with signature `sequenceForceBatches((bytes,bytes32,uint64,bytes32)[])` and selector `0x9f26f840`
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
        name = "sequenceForceBatches",
        abi = "sequenceForceBatches((bytes,bytes32,uint64,bytes32)[])"
    )]
    pub struct SequenceForceBatchesCall {
        pub batches: ::std::vec::Vec<BatchData>,
    }
    ///Container type for all input parameters for the `setForceBatchAddress` function with signature `setForceBatchAddress(address)` and selector `0x91cafe32`
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
    #[ethcall(name = "setForceBatchAddress", abi = "setForceBatchAddress(address)")]
    pub struct SetForceBatchAddressCall {
        pub new_force_batch_address: ::ethers::core::types::Address,
    }
    ///Container type for all input parameters for the `setForceBatchTimeout` function with signature `setForceBatchTimeout(uint64)` and selector `0x4e487706`
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
    #[ethcall(name = "setForceBatchTimeout", abi = "setForceBatchTimeout(uint64)")]
    pub struct SetForceBatchTimeoutCall {
        pub newforce_batch_timeout: u64,
    }
    ///Container type for all input parameters for the `setTrustedSequencer` function with signature `setTrustedSequencer(address)` and selector `0x6ff512cc`
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
    #[ethcall(name = "setTrustedSequencer", abi = "setTrustedSequencer(address)")]
    pub struct SetTrustedSequencerCall {
        pub new_trusted_sequencer: ::ethers::core::types::Address,
    }
    ///Container type for all input parameters for the `setTrustedSequencerURL` function with signature `setTrustedSequencerURL(string)` and selector `0xc89e42df`
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
    #[ethcall(name = "setTrustedSequencerURL", abi = "setTrustedSequencerURL(string)")]
    pub struct SetTrustedSequencerURLCall {
        pub new_trusted_sequencer_url: ::std::string::String,
    }
    ///Container type for all input parameters for the `transferAdminRole` function with signature `transferAdminRole(address)` and selector `0xada8f919`
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
    #[ethcall(name = "transferAdminRole", abi = "transferAdminRole(address)")]
    pub struct TransferAdminRoleCall {
        pub new_pending_admin: ::ethers::core::types::Address,
    }
    ///Container type for all input parameters for the `trustedSequencer` function with signature `trustedSequencer()` and selector `0xcfa8ed47`
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
    #[ethcall(name = "trustedSequencer", abi = "trustedSequencer()")]
    pub struct TrustedSequencerCall;
    ///Container type for all input parameters for the `trustedSequencerURL` function with signature `trustedSequencerURL()` and selector `0x542028d5`
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
    #[ethcall(name = "trustedSequencerURL", abi = "trustedSequencerURL()")]
    pub struct TrustedSequencerURLCall;
    ///Container type for all of the contract's call
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum PolygonZkEvmCalls {
        GlobalExitRootManagerL2(GlobalExitRootManagerL2Call),
        InitializeTxBridgeListLenLen(InitializeTxBridgeListLenLenCall),
        InitializeTxBridgeParams(InitializeTxBridgeParamsCall),
        InitializeTxBridgeParamsAfterBridgeAddress(
            InitializeTxBridgeParamsAfterBridgeAddressCall,
        ),
        InitializeTxBridgeParamsAfterBridgeAddressEmptyMetadata(
            InitializeTxBridgeParamsAfterBridgeAddressEmptyMetadataCall,
        ),
        InitializeTxConstantBytes(InitializeTxConstantBytesCall),
        InitializeTxConstantBytesEmptyMetadata(
            InitializeTxConstantBytesEmptyMetadataCall,
        ),
        InitializeTxDataLenEmptyMetadata(InitializeTxDataLenEmptyMetadataCall),
        InitializeTxEffectivePercentage(InitializeTxEffectivePercentageCall),
        SignatureInitializeTxR(SignatureInitializeTxRCall),
        SignatureInitializeTxS(SignatureInitializeTxSCall),
        SignatureInitializeTxV(SignatureInitializeTxVCall),
        TimestampRange(TimestampRangeCall),
        AcceptAdminRole(AcceptAdminRoleCall),
        Admin(AdminCall),
        BridgeAddress(BridgeAddressCall),
        CalculatePolPerForceBatch(CalculatePolPerForceBatchCall),
        ForceBatch(ForceBatchCall),
        ForceBatchAddress(ForceBatchAddressCall),
        ForceBatchTimeout(ForceBatchTimeoutCall),
        ForcedBatches(ForcedBatchesCall),
        GasTokenAddress(GasTokenAddressCall),
        GasTokenNetwork(GasTokenNetworkCall),
        GenerateInitializeTransaction(GenerateInitializeTransactionCall),
        GlobalExitRootManager(GlobalExitRootManagerCall),
        Initialize(InitializeCall),
        LastAccInputHash(LastAccInputHashCall),
        LastForceBatch(LastForceBatchCall),
        LastForceBatchSequenced(LastForceBatchSequencedCall),
        NetworkName(NetworkNameCall),
        OnVerifyBatches(OnVerifyBatchesCall),
        PendingAdmin(PendingAdminCall),
        Pol(PolCall),
        RollupManager(RollupManagerCall),
        SequenceBatches(SequenceBatchesCall),
        SequenceForceBatches(SequenceForceBatchesCall),
        SetForceBatchAddress(SetForceBatchAddressCall),
        SetForceBatchTimeout(SetForceBatchTimeoutCall),
        SetTrustedSequencer(SetTrustedSequencerCall),
        SetTrustedSequencerURL(SetTrustedSequencerURLCall),
        TransferAdminRole(TransferAdminRoleCall),
        TrustedSequencer(TrustedSequencerCall),
        TrustedSequencerURL(TrustedSequencerURLCall),
    }
    impl ::ethers::core::abi::AbiDecode for PolygonZkEvmCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded) = <GlobalExitRootManagerL2Call as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GlobalExitRootManagerL2(decoded));
            }
            if let Ok(decoded) = <InitializeTxBridgeListLenLenCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::InitializeTxBridgeListLenLen(decoded));
            }
            if let Ok(decoded) = <InitializeTxBridgeParamsCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::InitializeTxBridgeParams(decoded));
            }
            if let Ok(decoded) = <InitializeTxBridgeParamsAfterBridgeAddressCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::InitializeTxBridgeParamsAfterBridgeAddress(decoded));
            }
            if let Ok(decoded) = <InitializeTxBridgeParamsAfterBridgeAddressEmptyMetadataCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(
                    Self::InitializeTxBridgeParamsAfterBridgeAddressEmptyMetadata(
                        decoded,
                    ),
                );
            }
            if let Ok(decoded) = <InitializeTxConstantBytesCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::InitializeTxConstantBytes(decoded));
            }
            if let Ok(decoded) = <InitializeTxConstantBytesEmptyMetadataCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::InitializeTxConstantBytesEmptyMetadata(decoded));
            }
            if let Ok(decoded) = <InitializeTxDataLenEmptyMetadataCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::InitializeTxDataLenEmptyMetadata(decoded));
            }
            if let Ok(decoded) = <InitializeTxEffectivePercentageCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::InitializeTxEffectivePercentage(decoded));
            }
            if let Ok(decoded) = <SignatureInitializeTxRCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::SignatureInitializeTxR(decoded));
            }
            if let Ok(decoded) = <SignatureInitializeTxSCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::SignatureInitializeTxS(decoded));
            }
            if let Ok(decoded) = <SignatureInitializeTxVCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::SignatureInitializeTxV(decoded));
            }
            if let Ok(decoded) = <TimestampRangeCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::TimestampRange(decoded));
            }
            if let Ok(decoded) = <AcceptAdminRoleCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::AcceptAdminRole(decoded));
            }
            if let Ok(decoded) = <AdminCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::Admin(decoded));
            }
            if let Ok(decoded) = <BridgeAddressCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::BridgeAddress(decoded));
            }
            if let Ok(decoded) = <CalculatePolPerForceBatchCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::CalculatePolPerForceBatch(decoded));
            }
            if let Ok(decoded) = <ForceBatchCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::ForceBatch(decoded));
            }
            if let Ok(decoded) = <ForceBatchAddressCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::ForceBatchAddress(decoded));
            }
            if let Ok(decoded) = <ForceBatchTimeoutCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::ForceBatchTimeout(decoded));
            }
            if let Ok(decoded) = <ForcedBatchesCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::ForcedBatches(decoded));
            }
            if let Ok(decoded) = <GasTokenAddressCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GasTokenAddress(decoded));
            }
            if let Ok(decoded) = <GasTokenNetworkCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GasTokenNetwork(decoded));
            }
            if let Ok(decoded) = <GenerateInitializeTransactionCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GenerateInitializeTransaction(decoded));
            }
            if let Ok(decoded) = <GlobalExitRootManagerCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GlobalExitRootManager(decoded));
            }
            if let Ok(decoded) = <InitializeCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::Initialize(decoded));
            }
            if let Ok(decoded) = <LastAccInputHashCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::LastAccInputHash(decoded));
            }
            if let Ok(decoded) = <LastForceBatchCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::LastForceBatch(decoded));
            }
            if let Ok(decoded) = <LastForceBatchSequencedCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::LastForceBatchSequenced(decoded));
            }
            if let Ok(decoded) = <NetworkNameCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::NetworkName(decoded));
            }
            if let Ok(decoded) = <OnVerifyBatchesCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::OnVerifyBatches(decoded));
            }
            if let Ok(decoded) = <PendingAdminCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::PendingAdmin(decoded));
            }
            if let Ok(decoded) = <PolCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::Pol(decoded));
            }
            if let Ok(decoded) = <RollupManagerCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::RollupManager(decoded));
            }
            if let Ok(decoded) = <SequenceBatchesCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::SequenceBatches(decoded));
            }
            if let Ok(decoded) = <SequenceForceBatchesCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::SequenceForceBatches(decoded));
            }
            if let Ok(decoded) = <SetForceBatchAddressCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::SetForceBatchAddress(decoded));
            }
            if let Ok(decoded) = <SetForceBatchTimeoutCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::SetForceBatchTimeout(decoded));
            }
            if let Ok(decoded) = <SetTrustedSequencerCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::SetTrustedSequencer(decoded));
            }
            if let Ok(decoded) = <SetTrustedSequencerURLCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::SetTrustedSequencerURL(decoded));
            }
            if let Ok(decoded) = <TransferAdminRoleCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::TransferAdminRole(decoded));
            }
            if let Ok(decoded) = <TrustedSequencerCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::TrustedSequencer(decoded));
            }
            if let Ok(decoded) = <TrustedSequencerURLCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::TrustedSequencerURL(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for PolygonZkEvmCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                Self::GlobalExitRootManagerL2(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InitializeTxBridgeListLenLen(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InitializeTxBridgeParams(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InitializeTxBridgeParamsAfterBridgeAddress(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InitializeTxBridgeParamsAfterBridgeAddressEmptyMetadata(
                    element,
                ) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::InitializeTxConstantBytes(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InitializeTxConstantBytesEmptyMetadata(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InitializeTxDataLenEmptyMetadata(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InitializeTxEffectivePercentage(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::SignatureInitializeTxR(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::SignatureInitializeTxS(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::SignatureInitializeTxV(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::TimestampRange(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::AcceptAdminRole(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::Admin(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::BridgeAddress(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CalculatePolPerForceBatch(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ForceBatch(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ForceBatchAddress(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ForceBatchTimeout(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ForcedBatches(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GasTokenAddress(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GasTokenNetwork(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GenerateInitializeTransaction(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GlobalExitRootManager(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::Initialize(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::LastAccInputHash(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::LastForceBatch(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::LastForceBatchSequenced(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NetworkName(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::OnVerifyBatches(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::PendingAdmin(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::Pol(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::RollupManager(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::SequenceBatches(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::SequenceForceBatches(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::SetForceBatchAddress(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::SetForceBatchTimeout(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::SetTrustedSequencer(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::SetTrustedSequencerURL(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::TransferAdminRole(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::TrustedSequencer(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::TrustedSequencerURL(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
            }
        }
    }
    impl ::core::fmt::Display for PolygonZkEvmCalls {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::GlobalExitRootManagerL2(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InitializeTxBridgeListLenLen(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InitializeTxBridgeParams(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InitializeTxBridgeParamsAfterBridgeAddress(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InitializeTxBridgeParamsAfterBridgeAddressEmptyMetadata(
                    element,
                ) => ::core::fmt::Display::fmt(element, f),
                Self::InitializeTxConstantBytes(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InitializeTxConstantBytesEmptyMetadata(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InitializeTxDataLenEmptyMetadata(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InitializeTxEffectivePercentage(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::SignatureInitializeTxR(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::SignatureInitializeTxS(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::SignatureInitializeTxV(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::TimestampRange(element) => ::core::fmt::Display::fmt(element, f),
                Self::AcceptAdminRole(element) => ::core::fmt::Display::fmt(element, f),
                Self::Admin(element) => ::core::fmt::Display::fmt(element, f),
                Self::BridgeAddress(element) => ::core::fmt::Display::fmt(element, f),
                Self::CalculatePolPerForceBatch(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ForceBatch(element) => ::core::fmt::Display::fmt(element, f),
                Self::ForceBatchAddress(element) => ::core::fmt::Display::fmt(element, f),
                Self::ForceBatchTimeout(element) => ::core::fmt::Display::fmt(element, f),
                Self::ForcedBatches(element) => ::core::fmt::Display::fmt(element, f),
                Self::GasTokenAddress(element) => ::core::fmt::Display::fmt(element, f),
                Self::GasTokenNetwork(element) => ::core::fmt::Display::fmt(element, f),
                Self::GenerateInitializeTransaction(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::GlobalExitRootManager(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::Initialize(element) => ::core::fmt::Display::fmt(element, f),
                Self::LastAccInputHash(element) => ::core::fmt::Display::fmt(element, f),
                Self::LastForceBatch(element) => ::core::fmt::Display::fmt(element, f),
                Self::LastForceBatchSequenced(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::NetworkName(element) => ::core::fmt::Display::fmt(element, f),
                Self::OnVerifyBatches(element) => ::core::fmt::Display::fmt(element, f),
                Self::PendingAdmin(element) => ::core::fmt::Display::fmt(element, f),
                Self::Pol(element) => ::core::fmt::Display::fmt(element, f),
                Self::RollupManager(element) => ::core::fmt::Display::fmt(element, f),
                Self::SequenceBatches(element) => ::core::fmt::Display::fmt(element, f),
                Self::SequenceForceBatches(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::SetForceBatchAddress(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::SetForceBatchTimeout(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::SetTrustedSequencer(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::SetTrustedSequencerURL(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::TransferAdminRole(element) => ::core::fmt::Display::fmt(element, f),
                Self::TrustedSequencer(element) => ::core::fmt::Display::fmt(element, f),
                Self::TrustedSequencerURL(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
            }
        }
    }
    impl ::core::convert::From<GlobalExitRootManagerL2Call> for PolygonZkEvmCalls {
        fn from(value: GlobalExitRootManagerL2Call) -> Self {
            Self::GlobalExitRootManagerL2(value)
        }
    }
    impl ::core::convert::From<InitializeTxBridgeListLenLenCall> for PolygonZkEvmCalls {
        fn from(value: InitializeTxBridgeListLenLenCall) -> Self {
            Self::InitializeTxBridgeListLenLen(value)
        }
    }
    impl ::core::convert::From<InitializeTxBridgeParamsCall> for PolygonZkEvmCalls {
        fn from(value: InitializeTxBridgeParamsCall) -> Self {
            Self::InitializeTxBridgeParams(value)
        }
    }
    impl ::core::convert::From<InitializeTxBridgeParamsAfterBridgeAddressCall>
    for PolygonZkEvmCalls {
        fn from(value: InitializeTxBridgeParamsAfterBridgeAddressCall) -> Self {
            Self::InitializeTxBridgeParamsAfterBridgeAddress(value)
        }
    }
    impl ::core::convert::From<
        InitializeTxBridgeParamsAfterBridgeAddressEmptyMetadataCall,
    > for PolygonZkEvmCalls {
        fn from(
            value: InitializeTxBridgeParamsAfterBridgeAddressEmptyMetadataCall,
        ) -> Self {
            Self::InitializeTxBridgeParamsAfterBridgeAddressEmptyMetadata(value)
        }
    }
    impl ::core::convert::From<InitializeTxConstantBytesCall> for PolygonZkEvmCalls {
        fn from(value: InitializeTxConstantBytesCall) -> Self {
            Self::InitializeTxConstantBytes(value)
        }
    }
    impl ::core::convert::From<InitializeTxConstantBytesEmptyMetadataCall>
    for PolygonZkEvmCalls {
        fn from(value: InitializeTxConstantBytesEmptyMetadataCall) -> Self {
            Self::InitializeTxConstantBytesEmptyMetadata(value)
        }
    }
    impl ::core::convert::From<InitializeTxDataLenEmptyMetadataCall>
    for PolygonZkEvmCalls {
        fn from(value: InitializeTxDataLenEmptyMetadataCall) -> Self {
            Self::InitializeTxDataLenEmptyMetadata(value)
        }
    }
    impl ::core::convert::From<InitializeTxEffectivePercentageCall>
    for PolygonZkEvmCalls {
        fn from(value: InitializeTxEffectivePercentageCall) -> Self {
            Self::InitializeTxEffectivePercentage(value)
        }
    }
    impl ::core::convert::From<SignatureInitializeTxRCall> for PolygonZkEvmCalls {
        fn from(value: SignatureInitializeTxRCall) -> Self {
            Self::SignatureInitializeTxR(value)
        }
    }
    impl ::core::convert::From<SignatureInitializeTxSCall> for PolygonZkEvmCalls {
        fn from(value: SignatureInitializeTxSCall) -> Self {
            Self::SignatureInitializeTxS(value)
        }
    }
    impl ::core::convert::From<SignatureInitializeTxVCall> for PolygonZkEvmCalls {
        fn from(value: SignatureInitializeTxVCall) -> Self {
            Self::SignatureInitializeTxV(value)
        }
    }
    impl ::core::convert::From<TimestampRangeCall> for PolygonZkEvmCalls {
        fn from(value: TimestampRangeCall) -> Self {
            Self::TimestampRange(value)
        }
    }
    impl ::core::convert::From<AcceptAdminRoleCall> for PolygonZkEvmCalls {
        fn from(value: AcceptAdminRoleCall) -> Self {
            Self::AcceptAdminRole(value)
        }
    }
    impl ::core::convert::From<AdminCall> for PolygonZkEvmCalls {
        fn from(value: AdminCall) -> Self {
            Self::Admin(value)
        }
    }
    impl ::core::convert::From<BridgeAddressCall> for PolygonZkEvmCalls {
        fn from(value: BridgeAddressCall) -> Self {
            Self::BridgeAddress(value)
        }
    }
    impl ::core::convert::From<CalculatePolPerForceBatchCall> for PolygonZkEvmCalls {
        fn from(value: CalculatePolPerForceBatchCall) -> Self {
            Self::CalculatePolPerForceBatch(value)
        }
    }
    impl ::core::convert::From<ForceBatchCall> for PolygonZkEvmCalls {
        fn from(value: ForceBatchCall) -> Self {
            Self::ForceBatch(value)
        }
    }
    impl ::core::convert::From<ForceBatchAddressCall> for PolygonZkEvmCalls {
        fn from(value: ForceBatchAddressCall) -> Self {
            Self::ForceBatchAddress(value)
        }
    }
    impl ::core::convert::From<ForceBatchTimeoutCall> for PolygonZkEvmCalls {
        fn from(value: ForceBatchTimeoutCall) -> Self {
            Self::ForceBatchTimeout(value)
        }
    }
    impl ::core::convert::From<ForcedBatchesCall> for PolygonZkEvmCalls {
        fn from(value: ForcedBatchesCall) -> Self {
            Self::ForcedBatches(value)
        }
    }
    impl ::core::convert::From<GasTokenAddressCall> for PolygonZkEvmCalls {
        fn from(value: GasTokenAddressCall) -> Self {
            Self::GasTokenAddress(value)
        }
    }
    impl ::core::convert::From<GasTokenNetworkCall> for PolygonZkEvmCalls {
        fn from(value: GasTokenNetworkCall) -> Self {
            Self::GasTokenNetwork(value)
        }
    }
    impl ::core::convert::From<GenerateInitializeTransactionCall> for PolygonZkEvmCalls {
        fn from(value: GenerateInitializeTransactionCall) -> Self {
            Self::GenerateInitializeTransaction(value)
        }
    }
    impl ::core::convert::From<GlobalExitRootManagerCall> for PolygonZkEvmCalls {
        fn from(value: GlobalExitRootManagerCall) -> Self {
            Self::GlobalExitRootManager(value)
        }
    }
    impl ::core::convert::From<InitializeCall> for PolygonZkEvmCalls {
        fn from(value: InitializeCall) -> Self {
            Self::Initialize(value)
        }
    }
    impl ::core::convert::From<LastAccInputHashCall> for PolygonZkEvmCalls {
        fn from(value: LastAccInputHashCall) -> Self {
            Self::LastAccInputHash(value)
        }
    }
    impl ::core::convert::From<LastForceBatchCall> for PolygonZkEvmCalls {
        fn from(value: LastForceBatchCall) -> Self {
            Self::LastForceBatch(value)
        }
    }
    impl ::core::convert::From<LastForceBatchSequencedCall> for PolygonZkEvmCalls {
        fn from(value: LastForceBatchSequencedCall) -> Self {
            Self::LastForceBatchSequenced(value)
        }
    }
    impl ::core::convert::From<NetworkNameCall> for PolygonZkEvmCalls {
        fn from(value: NetworkNameCall) -> Self {
            Self::NetworkName(value)
        }
    }
    impl ::core::convert::From<OnVerifyBatchesCall> for PolygonZkEvmCalls {
        fn from(value: OnVerifyBatchesCall) -> Self {
            Self::OnVerifyBatches(value)
        }
    }
    impl ::core::convert::From<PendingAdminCall> for PolygonZkEvmCalls {
        fn from(value: PendingAdminCall) -> Self {
            Self::PendingAdmin(value)
        }
    }
    impl ::core::convert::From<PolCall> for PolygonZkEvmCalls {
        fn from(value: PolCall) -> Self {
            Self::Pol(value)
        }
    }
    impl ::core::convert::From<RollupManagerCall> for PolygonZkEvmCalls {
        fn from(value: RollupManagerCall) -> Self {
            Self::RollupManager(value)
        }
    }
    impl ::core::convert::From<SequenceBatchesCall> for PolygonZkEvmCalls {
        fn from(value: SequenceBatchesCall) -> Self {
            Self::SequenceBatches(value)
        }
    }
    impl ::core::convert::From<SequenceForceBatchesCall> for PolygonZkEvmCalls {
        fn from(value: SequenceForceBatchesCall) -> Self {
            Self::SequenceForceBatches(value)
        }
    }
    impl ::core::convert::From<SetForceBatchAddressCall> for PolygonZkEvmCalls {
        fn from(value: SetForceBatchAddressCall) -> Self {
            Self::SetForceBatchAddress(value)
        }
    }
    impl ::core::convert::From<SetForceBatchTimeoutCall> for PolygonZkEvmCalls {
        fn from(value: SetForceBatchTimeoutCall) -> Self {
            Self::SetForceBatchTimeout(value)
        }
    }
    impl ::core::convert::From<SetTrustedSequencerCall> for PolygonZkEvmCalls {
        fn from(value: SetTrustedSequencerCall) -> Self {
            Self::SetTrustedSequencer(value)
        }
    }
    impl ::core::convert::From<SetTrustedSequencerURLCall> for PolygonZkEvmCalls {
        fn from(value: SetTrustedSequencerURLCall) -> Self {
            Self::SetTrustedSequencerURL(value)
        }
    }
    impl ::core::convert::From<TransferAdminRoleCall> for PolygonZkEvmCalls {
        fn from(value: TransferAdminRoleCall) -> Self {
            Self::TransferAdminRole(value)
        }
    }
    impl ::core::convert::From<TrustedSequencerCall> for PolygonZkEvmCalls {
        fn from(value: TrustedSequencerCall) -> Self {
            Self::TrustedSequencer(value)
        }
    }
    impl ::core::convert::From<TrustedSequencerURLCall> for PolygonZkEvmCalls {
        fn from(value: TrustedSequencerURLCall) -> Self {
            Self::TrustedSequencerURL(value)
        }
    }
    ///Container type for all return fields from the `GLOBAL_EXIT_ROOT_MANAGER_L2` function with signature `GLOBAL_EXIT_ROOT_MANAGER_L2()` and selector `0x9e001877`
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
    pub struct GlobalExitRootManagerL2Return(pub ::ethers::core::types::Address);
    ///Container type for all return fields from the `INITIALIZE_TX_BRIDGE_LIST_LEN_LEN` function with signature `INITIALIZE_TX_BRIDGE_LIST_LEN_LEN()` and selector `0x11e892d4`
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
    pub struct InitializeTxBridgeListLenLenReturn(pub u8);
    ///Container type for all return fields from the `INITIALIZE_TX_BRIDGE_PARAMS` function with signature `INITIALIZE_TX_BRIDGE_PARAMS()` and selector `0x05835f37`
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
    pub struct InitializeTxBridgeParamsReturn(pub ::ethers::core::types::Bytes);
    ///Container type for all return fields from the `INITIALIZE_TX_BRIDGE_PARAMS_AFTER_BRIDGE_ADDRESS` function with signature `INITIALIZE_TX_BRIDGE_PARAMS_AFTER_BRIDGE_ADDRESS()` and selector `0x7a5460c5`
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
    pub struct InitializeTxBridgeParamsAfterBridgeAddressReturn(
        pub ::ethers::core::types::Bytes,
    );
    ///Container type for all return fields from the `INITIALIZE_TX_BRIDGE_PARAMS_AFTER_BRIDGE_ADDRESS_EMPTY_METADATA` function with signature `INITIALIZE_TX_BRIDGE_PARAMS_AFTER_BRIDGE_ADDRESS_EMPTY_METADATA()` and selector `0x52bdeb6d`
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
    pub struct InitializeTxBridgeParamsAfterBridgeAddressEmptyMetadataReturn(
        pub ::ethers::core::types::Bytes,
    );
    ///Container type for all return fields from the `INITIALIZE_TX_CONSTANT_BYTES` function with signature `INITIALIZE_TX_CONSTANT_BYTES()` and selector `0x03508963`
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
    pub struct InitializeTxConstantBytesReturn(pub u16);
    ///Container type for all return fields from the `INITIALIZE_TX_CONSTANT_BYTES_EMPTY_METADATA` function with signature `INITIALIZE_TX_CONSTANT_BYTES_EMPTY_METADATA()` and selector `0x676870d2`
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
    pub struct InitializeTxConstantBytesEmptyMetadataReturn(pub u16);
    ///Container type for all return fields from the `INITIALIZE_TX_DATA_LEN_EMPTY_METADATA` function with signature `INITIALIZE_TX_DATA_LEN_EMPTY_METADATA()` and selector `0xc7fffd4b`
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
    pub struct InitializeTxDataLenEmptyMetadataReturn(pub u8);
    ///Container type for all return fields from the `INITIALIZE_TX_EFFECTIVE_PERCENTAGE` function with signature `INITIALIZE_TX_EFFECTIVE_PERCENTAGE()` and selector `0x40b5de6c`
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
    pub struct InitializeTxEffectivePercentageReturn(pub [u8; 1]);
    ///Container type for all return fields from the `SIGNATURE_INITIALIZE_TX_R` function with signature `SIGNATURE_INITIALIZE_TX_R()` and selector `0xb0afe154`
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
    pub struct SignatureInitializeTxRReturn(pub [u8; 32]);
    ///Container type for all return fields from the `SIGNATURE_INITIALIZE_TX_S` function with signature `SIGNATURE_INITIALIZE_TX_S()` and selector `0xd7bc90ff`
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
    pub struct SignatureInitializeTxSReturn(pub [u8; 32]);
    ///Container type for all return fields from the `SIGNATURE_INITIALIZE_TX_V` function with signature `SIGNATURE_INITIALIZE_TX_V()` and selector `0xf35dda47`
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
    pub struct SignatureInitializeTxVReturn(pub u8);
    ///Container type for all return fields from the `TIMESTAMP_RANGE` function with signature `TIMESTAMP_RANGE()` and selector `0x42308fab`
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
    pub struct TimestampRangeReturn(pub ::ethers::core::types::U256);
    ///Container type for all return fields from the `admin` function with signature `admin()` and selector `0xf851a440`
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
    pub struct AdminReturn(pub ::ethers::core::types::Address);
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
    ///Container type for all return fields from the `calculatePolPerForceBatch` function with signature `calculatePolPerForceBatch()` and selector `0x00d0295d`
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
    pub struct CalculatePolPerForceBatchReturn(pub ::ethers::core::types::U256);
    ///Container type for all return fields from the `forceBatchAddress` function with signature `forceBatchAddress()` and selector `0x2c111c06`
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
    pub struct ForceBatchAddressReturn(pub ::ethers::core::types::Address);
    ///Container type for all return fields from the `forceBatchTimeout` function with signature `forceBatchTimeout()` and selector `0xc754c7ed`
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
    pub struct ForceBatchTimeoutReturn(pub u64);
    ///Container type for all return fields from the `forcedBatches` function with signature `forcedBatches(uint64)` and selector `0x6b8616ce`
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
    pub struct ForcedBatchesReturn(pub [u8; 32]);
    ///Container type for all return fields from the `gasTokenAddress` function with signature `gasTokenAddress()` and selector `0x3c351e10`
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
    pub struct GasTokenAddressReturn(pub ::ethers::core::types::Address);
    ///Container type for all return fields from the `gasTokenNetwork` function with signature `gasTokenNetwork()` and selector `0x3cbc795b`
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
    pub struct GasTokenNetworkReturn(pub u32);
    ///Container type for all return fields from the `generateInitializeTransaction` function with signature `generateInitializeTransaction(uint32,address,uint32,bytes)` and selector `0xa652f26c`
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
    pub struct GenerateInitializeTransactionReturn(pub ::ethers::core::types::Bytes);
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
    ///Container type for all return fields from the `lastAccInputHash` function with signature `lastAccInputHash()` and selector `0x6e05d2cd`
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
    pub struct LastAccInputHashReturn(pub [u8; 32]);
    ///Container type for all return fields from the `lastForceBatch` function with signature `lastForceBatch()` and selector `0xe7a7ed02`
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
    pub struct LastForceBatchReturn(pub u64);
    ///Container type for all return fields from the `lastForceBatchSequenced` function with signature `lastForceBatchSequenced()` and selector `0x45605267`
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
    pub struct LastForceBatchSequencedReturn(pub u64);
    ///Container type for all return fields from the `networkName` function with signature `networkName()` and selector `0x107bf28c`
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
    pub struct NetworkNameReturn(pub ::std::string::String);
    ///Container type for all return fields from the `pendingAdmin` function with signature `pendingAdmin()` and selector `0x26782247`
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
    pub struct PendingAdminReturn(pub ::ethers::core::types::Address);
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
    ///Container type for all return fields from the `rollupManager` function with signature `rollupManager()` and selector `0x49b7b802`
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
    pub struct RollupManagerReturn(pub ::ethers::core::types::Address);
    ///Container type for all return fields from the `trustedSequencer` function with signature `trustedSequencer()` and selector `0xcfa8ed47`
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
    pub struct TrustedSequencerReturn(pub ::ethers::core::types::Address);
    ///Container type for all return fields from the `trustedSequencerURL` function with signature `trustedSequencerURL()` and selector `0x542028d5`
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
    pub struct TrustedSequencerURLReturn(pub ::std::string::String);
    ///`BatchData(bytes,bytes32,uint64,bytes32)`
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
    pub struct BatchData {
        pub transactions: ::ethers::core::types::Bytes,
        pub forced_global_exit_root: [u8; 32],
        pub forced_timestamp: u64,
        pub forced_block_hash_l1: [u8; 32],
    }
}
