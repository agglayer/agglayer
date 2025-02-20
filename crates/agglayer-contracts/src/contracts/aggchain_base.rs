pub use aggchain_base::*;
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
pub mod aggchain_base {
    #[allow(deprecated)]
    fn __abi() -> ::ethers::core::abi::Abi {
        ::ethers::core::abi::ethabi::Contract {
            constructor: ::core::option::Option::None,
            functions: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("AGGCHAIN_TYPE"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("AGGCHAIN_TYPE"),
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
                    ::std::borrow::ToOwned::to_owned("acceptVKeyManagerRole"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "acceptVKeyManagerRole",
                            ),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("addOwnedAggchainVKey"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "addOwnedAggchainVKey",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("aggchainSelector"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        4usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes4"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("newAggchainVKey"),
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
                                        ::std::borrow::ToOwned::to_owned(
                                            "contract IAggLayerGateway",
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
                    ::std::borrow::ToOwned::to_owned("disableUseDefaultGatewayFlag"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "disableUseDefaultGatewayFlag",
                            ),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("enableUseDefaultGatewayFlag"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "enableUseDefaultGatewayFlag",
                            ),
                            inputs: ::std::vec![],
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
                    ::std::borrow::ToOwned::to_owned("getAggchainVKey"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("getAggchainVKey"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("aggchainSelector"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        4usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes4"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("aggchainVKey"),
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
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::Pure,
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
                    ::std::borrow::ToOwned::to_owned("ownedAggchainVKeys"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("ownedAggchainVKeys"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "aggchainVKeySelector",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        4usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes4"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("ownedAggchainVKey"),
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
                    ::std::borrow::ToOwned::to_owned("pendingVKeyManager"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("pendingVKeyManager"),
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
                    ::std::borrow::ToOwned::to_owned("transferVKeyManagerRole"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "transferVKeyManagerRole",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("newVKeyManager"),
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
                (
                    ::std::borrow::ToOwned::to_owned("updateOwnedAggchainVKey"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "updateOwnedAggchainVKey",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("aggchainSelector"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        4usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes4"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "updatedAggchainVKey",
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
                    ::std::borrow::ToOwned::to_owned("useDefaultGateway"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("useDefaultGateway"),
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
                    ::std::borrow::ToOwned::to_owned("vKeyManager"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("vKeyManager"),
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
                    ::std::borrow::ToOwned::to_owned("AcceptVKeyManagerRole"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned(
                                "AcceptVKeyManagerRole",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("newVKeyManager"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("AddAggchainVKey"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("AddAggchainVKey"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("selector"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        4usize,
                                    ),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("newAggchainVKey"),
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
                    ::std::borrow::ToOwned::to_owned("TransferVKeyManagerRole"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned(
                                "TransferVKeyManagerRole",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("newVKeyManager"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("UpdateAggchainVKey"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("UpdateAggchainVKey"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("selector"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        4usize,
                                    ),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("newAggchainVKey"),
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
                    ::std::borrow::ToOwned::to_owned("UpdateUseDefaultGatewayFlag"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned(
                                "UpdateUseDefaultGatewayFlag",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("useDefaultGateway"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bool,
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
            ]),
            errors: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("AggchainVKeyNotFound"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "AggchainVKeyNotFound",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
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
                    ::std::borrow::ToOwned::to_owned("FinalAccInputHashDoesNotMatch"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "FinalAccInputHashDoesNotMatch",
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
                    ::std::borrow::ToOwned::to_owned("InvalidAggchainVKey"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "InvalidAggchainVKey",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("InvalidInitializeFunction"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "InvalidInitializeFunction",
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
                    ::std::borrow::ToOwned::to_owned("InvalidInitializer"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("InvalidInitializer"),
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
                    ::std::borrow::ToOwned::to_owned("OnlyPendingVKeyManager"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "OnlyPendingVKeyManager",
                            ),
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
                    ::std::borrow::ToOwned::to_owned("OnlyVKeyManager"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("OnlyVKeyManager"),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("OwnedAggchainVKeyAlreadyAdded"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "OwnedAggchainVKeyAlreadyAdded",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("OwnedAggchainVKeyLengthMismatch"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "OwnedAggchainVKeyLengthMismatch",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("OwnedAggchainVKeyNotFound"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "OwnedAggchainVKeyNotFound",
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
                (
                    ::std::borrow::ToOwned::to_owned("UseDefaultGatewayAlreadySet"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "UseDefaultGatewayAlreadySet",
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
    pub static AGGCHAINBASE_ABI: ::ethers::contract::Lazy<::ethers::core::abi::Abi> = ::ethers::contract::Lazy::new(
        __abi,
    );
    pub struct AggchainBase<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for AggchainBase<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for AggchainBase<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for AggchainBase<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for AggchainBase<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(::core::stringify!(AggchainBase))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> AggchainBase<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(
                ::ethers::contract::Contract::new(
                    address.into(),
                    AGGCHAINBASE_ABI.clone(),
                    client,
                ),
            )
        }
        ///Calls the contract's `AGGCHAIN_TYPE` (0x6e7fbce9) function
        pub fn aggchain_type(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, u32> {
            self.0
                .method_hash([110, 127, 188, 233], ())
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
        ///Calls the contract's `acceptVKeyManagerRole` (0x368c822c) function
        pub fn accept_v_key_manager_role(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([54, 140, 130, 44], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `addOwnedAggchainVKey` (0x19451a8f) function
        pub fn add_owned_aggchain_v_key(
            &self,
            aggchain_selector: [u8; 4],
            new_aggchain_v_key: [u8; 32],
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([25, 69, 26, 143], (aggchain_selector, new_aggchain_v_key))
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
        ///Calls the contract's `disableUseDefaultGatewayFlag` (0xdc8c4249) function
        pub fn disable_use_default_gateway_flag(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([220, 140, 66, 73], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `enableUseDefaultGatewayFlag` (0xe631476c) function
        pub fn enable_use_default_gateway_flag(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([230, 49, 71, 108], ())
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
        ///Calls the contract's `getAggchainVKey` (0x01fcf6a0) function
        pub fn get_aggchain_v_key(
            &self,
            aggchain_selector: [u8; 4],
        ) -> ::ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([1, 252, 246, 160], aggchain_selector)
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
            p0: ::ethers::core::types::Address,
            p1: ::ethers::core::types::Address,
            p2: u32,
            p3: ::ethers::core::types::Address,
            p4: ::std::string::String,
            p5: ::std::string::String,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([113, 37, 112, 34], (p0, p1, p2, p3, p4, p5))
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
        ///Calls the contract's `ownedAggchainVKeys` (0xeffb8479) function
        pub fn owned_aggchain_v_keys(
            &self,
            aggchain_v_key_selector: [u8; 4],
        ) -> ::ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([239, 251, 132, 121], aggchain_v_key_selector)
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
        ///Calls the contract's `pendingVKeyManager` (0xbfb193b6) function
        pub fn pending_v_key_manager(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Address,
        > {
            self.0
                .method_hash([191, 177, 147, 182], ())
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
        ///Calls the contract's `transferVKeyManagerRole` (0x85018182) function
        pub fn transfer_v_key_manager_role(
            &self,
            new_v_key_manager: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([133, 1, 129, 130], new_v_key_manager)
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
        ///Calls the contract's `updateOwnedAggchainVKey` (0x314eb17b) function
        pub fn update_owned_aggchain_v_key(
            &self,
            aggchain_selector: [u8; 4],
            updated_aggchain_v_key: [u8; 32],
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [49, 78, 177, 123],
                    (aggchain_selector, updated_aggchain_v_key),
                )
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `useDefaultGateway` (0xff904079) function
        pub fn use_default_gateway(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([255, 144, 64, 121], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `vKeyManager` (0xe279984e) function
        pub fn v_key_manager(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Address,
        > {
            self.0
                .method_hash([226, 121, 152, 78], ())
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
        ///Gets the contract's `AcceptVKeyManagerRole` event
        pub fn accept_v_key_manager_role_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            AcceptVKeyManagerRoleFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `AddAggchainVKey` event
        pub fn add_aggchain_v_key_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            AddAggchainVKeyFilter,
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
        ///Gets the contract's `TransferVKeyManagerRole` event
        pub fn transfer_v_key_manager_role_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            TransferVKeyManagerRoleFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `UpdateAggchainVKey` event
        pub fn update_aggchain_v_key_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            UpdateAggchainVKeyFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `UpdateUseDefaultGatewayFlag` event
        pub fn update_use_default_gateway_flag_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            UpdateUseDefaultGatewayFlagFilter,
        > {
            self.0.event()
        }
        /// Returns an `Event` builder for all the events of this contract.
        pub fn events(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            AggchainBaseEvents,
        > {
            self.0.event_with_filter(::core::default::Default::default())
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
    for AggchainBase<M> {
        fn from(contract: ::ethers::contract::Contract<M>) -> Self {
            Self::new(contract.address(), contract.client())
        }
    }
    ///Custom Error type `AggchainVKeyNotFound` with signature `AggchainVKeyNotFound()` and selector `0x925e5a3a`
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
    #[etherror(name = "AggchainVKeyNotFound", abi = "AggchainVKeyNotFound()")]
    pub struct AggchainVKeyNotFound;
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
    ///Custom Error type `FinalAccInputHashDoesNotMatch` with signature `FinalAccInputHashDoesNotMatch()` and selector `0xda5bceb9`
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
        name = "FinalAccInputHashDoesNotMatch",
        abi = "FinalAccInputHashDoesNotMatch()"
    )]
    pub struct FinalAccInputHashDoesNotMatch;
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
    ///Custom Error type `InvalidAggchainVKey` with signature `InvalidAggchainVKey()` and selector `0x4aac8b88`
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
    #[etherror(name = "InvalidAggchainVKey", abi = "InvalidAggchainVKey()")]
    pub struct InvalidAggchainVKey;
    ///Custom Error type `InvalidInitializeFunction` with signature `InvalidInitializeFunction()` and selector `0xf57ac683`
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
    #[etherror(name = "InvalidInitializeFunction", abi = "InvalidInitializeFunction()")]
    pub struct InvalidInitializeFunction;
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
    ///Custom Error type `InvalidInitializer` with signature `InvalidInitializer()` and selector `0xadc06ae7`
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
    #[etherror(name = "InvalidInitializer", abi = "InvalidInitializer()")]
    pub struct InvalidInitializer;
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
    ///Custom Error type `OnlyPendingVKeyManager` with signature `OnlyPendingVKeyManager()` and selector `0x05882cf0`
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
    #[etherror(name = "OnlyPendingVKeyManager", abi = "OnlyPendingVKeyManager()")]
    pub struct OnlyPendingVKeyManager;
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
    ///Custom Error type `OnlyVKeyManager` with signature `OnlyVKeyManager()` and selector `0xe4d753bd`
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
    #[etherror(name = "OnlyVKeyManager", abi = "OnlyVKeyManager()")]
    pub struct OnlyVKeyManager;
    ///Custom Error type `OwnedAggchainVKeyAlreadyAdded` with signature `OwnedAggchainVKeyAlreadyAdded()` and selector `0xe3cc7610`
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
        name = "OwnedAggchainVKeyAlreadyAdded",
        abi = "OwnedAggchainVKeyAlreadyAdded()"
    )]
    pub struct OwnedAggchainVKeyAlreadyAdded;
    ///Custom Error type `OwnedAggchainVKeyLengthMismatch` with signature `OwnedAggchainVKeyLengthMismatch()` and selector `0x68965113`
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
        name = "OwnedAggchainVKeyLengthMismatch",
        abi = "OwnedAggchainVKeyLengthMismatch()"
    )]
    pub struct OwnedAggchainVKeyLengthMismatch;
    ///Custom Error type `OwnedAggchainVKeyNotFound` with signature `OwnedAggchainVKeyNotFound()` and selector `0xf360deaf`
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
    #[etherror(name = "OwnedAggchainVKeyNotFound", abi = "OwnedAggchainVKeyNotFound()")]
    pub struct OwnedAggchainVKeyNotFound;
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
    ///Custom Error type `UseDefaultGatewayAlreadySet` with signature `UseDefaultGatewayAlreadySet()` and selector `0x6f318e4c`
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
        name = "UseDefaultGatewayAlreadySet",
        abi = "UseDefaultGatewayAlreadySet()"
    )]
    pub struct UseDefaultGatewayAlreadySet;
    ///Container type for all of the contract's custom errors
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum AggchainBaseErrors {
        AggchainVKeyNotFound(AggchainVKeyNotFound),
        BatchAlreadyVerified(BatchAlreadyVerified),
        BatchNotSequencedOrNotSequenceEnd(BatchNotSequencedOrNotSequenceEnd),
        ExceedMaxVerifyBatches(ExceedMaxVerifyBatches),
        FinalAccInputHashDoesNotMatch(FinalAccInputHashDoesNotMatch),
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
        InvalidAggchainVKey(InvalidAggchainVKey),
        InvalidInitializeFunction(InvalidInitializeFunction),
        InvalidInitializeTransaction(InvalidInitializeTransaction),
        InvalidInitializer(InvalidInitializer),
        InvalidProof(InvalidProof),
        InvalidRangeBatchTimeTarget(InvalidRangeBatchTimeTarget),
        InvalidRangeForceBatchTimeout(InvalidRangeForceBatchTimeout),
        InvalidRangeMultiplierBatchFee(InvalidRangeMultiplierBatchFee),
        L1InfoTreeLeafCountInvalid(L1InfoTreeLeafCountInvalid),
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
        OnlyPendingVKeyManager(OnlyPendingVKeyManager),
        OnlyRollupManager(OnlyRollupManager),
        OnlyTrustedAggregator(OnlyTrustedAggregator),
        OnlyTrustedSequencer(OnlyTrustedSequencer),
        OnlyVKeyManager(OnlyVKeyManager),
        OwnedAggchainVKeyAlreadyAdded(OwnedAggchainVKeyAlreadyAdded),
        OwnedAggchainVKeyLengthMismatch(OwnedAggchainVKeyLengthMismatch),
        OwnedAggchainVKeyNotFound(OwnedAggchainVKeyNotFound),
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
        UseDefaultGatewayAlreadySet(UseDefaultGatewayAlreadySet),
        /// The standard solidity revert string, with selector
        /// Error(string) -- 0x08c379a0
        RevertString(::std::string::String),
    }
    impl ::ethers::core::abi::AbiDecode for AggchainBaseErrors {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded) = <::std::string::String as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::RevertString(decoded));
            }
            if let Ok(decoded) = <AggchainVKeyNotFound as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::AggchainVKeyNotFound(decoded));
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
            if let Ok(decoded) = <FinalAccInputHashDoesNotMatch as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::FinalAccInputHashDoesNotMatch(decoded));
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
            if let Ok(decoded) = <InvalidAggchainVKey as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::InvalidAggchainVKey(decoded));
            }
            if let Ok(decoded) = <InvalidInitializeFunction as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::InvalidInitializeFunction(decoded));
            }
            if let Ok(decoded) = <InvalidInitializeTransaction as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::InvalidInitializeTransaction(decoded));
            }
            if let Ok(decoded) = <InvalidInitializer as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::InvalidInitializer(decoded));
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
            if let Ok(decoded) = <L1InfoTreeLeafCountInvalid as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::L1InfoTreeLeafCountInvalid(decoded));
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
            if let Ok(decoded) = <OnlyPendingVKeyManager as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::OnlyPendingVKeyManager(decoded));
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
            if let Ok(decoded) = <OnlyVKeyManager as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::OnlyVKeyManager(decoded));
            }
            if let Ok(decoded) = <OwnedAggchainVKeyAlreadyAdded as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::OwnedAggchainVKeyAlreadyAdded(decoded));
            }
            if let Ok(decoded) = <OwnedAggchainVKeyLengthMismatch as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::OwnedAggchainVKeyLengthMismatch(decoded));
            }
            if let Ok(decoded) = <OwnedAggchainVKeyNotFound as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::OwnedAggchainVKeyNotFound(decoded));
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
            if let Ok(decoded) = <UseDefaultGatewayAlreadySet as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::UseDefaultGatewayAlreadySet(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for AggchainBaseErrors {
        fn encode(self) -> ::std::vec::Vec<u8> {
            match self {
                Self::AggchainVKeyNotFound(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::BatchAlreadyVerified(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::BatchNotSequencedOrNotSequenceEnd(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ExceedMaxVerifyBatches(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::FinalAccInputHashDoesNotMatch(element) => {
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
                Self::InvalidAggchainVKey(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidInitializeFunction(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidInitializeTransaction(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidInitializer(element) => {
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
                Self::L1InfoTreeLeafCountInvalid(element) => {
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
                Self::OnlyPendingVKeyManager(element) => {
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
                Self::OnlyVKeyManager(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::OwnedAggchainVKeyAlreadyAdded(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::OwnedAggchainVKeyLengthMismatch(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::OwnedAggchainVKeyNotFound(element) => {
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
                Self::UseDefaultGatewayAlreadySet(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RevertString(s) => ::ethers::core::abi::AbiEncode::encode(s),
            }
        }
    }
    impl ::ethers::contract::ContractRevert for AggchainBaseErrors {
        fn valid_selector(selector: [u8; 4]) -> bool {
            match selector {
                [0x08, 0xc3, 0x79, 0xa0] => true,
                _ if selector
                    == <AggchainVKeyNotFound as ::ethers::contract::EthError>::selector() => {
                    true
                }
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
                    == <FinalAccInputHashDoesNotMatch as ::ethers::contract::EthError>::selector() => {
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
                    == <InvalidAggchainVKey as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InvalidInitializeFunction as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InvalidInitializeTransaction as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InvalidInitializer as ::ethers::contract::EthError>::selector() => {
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
                    == <L1InfoTreeLeafCountInvalid as ::ethers::contract::EthError>::selector() => {
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
                    == <OnlyPendingVKeyManager as ::ethers::contract::EthError>::selector() => {
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
                    == <OnlyVKeyManager as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <OwnedAggchainVKeyAlreadyAdded as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <OwnedAggchainVKeyLengthMismatch as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <OwnedAggchainVKeyNotFound as ::ethers::contract::EthError>::selector() => {
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
                _ if selector
                    == <UseDefaultGatewayAlreadySet as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ => false,
            }
        }
    }
    impl ::core::fmt::Display for AggchainBaseErrors {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::AggchainVKeyNotFound(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::BatchAlreadyVerified(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::BatchNotSequencedOrNotSequenceEnd(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ExceedMaxVerifyBatches(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::FinalAccInputHashDoesNotMatch(element) => {
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
                Self::InvalidAggchainVKey(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InvalidInitializeFunction(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InvalidInitializeTransaction(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InvalidInitializer(element) => {
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
                Self::L1InfoTreeLeafCountInvalid(element) => {
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
                Self::OnlyPendingVKeyManager(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::OnlyRollupManager(element) => ::core::fmt::Display::fmt(element, f),
                Self::OnlyTrustedAggregator(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::OnlyTrustedSequencer(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::OnlyVKeyManager(element) => ::core::fmt::Display::fmt(element, f),
                Self::OwnedAggchainVKeyAlreadyAdded(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::OwnedAggchainVKeyLengthMismatch(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::OwnedAggchainVKeyNotFound(element) => {
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
                Self::UseDefaultGatewayAlreadySet(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RevertString(s) => ::core::fmt::Display::fmt(s, f),
            }
        }
    }
    impl ::core::convert::From<::std::string::String> for AggchainBaseErrors {
        fn from(value: String) -> Self {
            Self::RevertString(value)
        }
    }
    impl ::core::convert::From<AggchainVKeyNotFound> for AggchainBaseErrors {
        fn from(value: AggchainVKeyNotFound) -> Self {
            Self::AggchainVKeyNotFound(value)
        }
    }
    impl ::core::convert::From<BatchAlreadyVerified> for AggchainBaseErrors {
        fn from(value: BatchAlreadyVerified) -> Self {
            Self::BatchAlreadyVerified(value)
        }
    }
    impl ::core::convert::From<BatchNotSequencedOrNotSequenceEnd>
    for AggchainBaseErrors {
        fn from(value: BatchNotSequencedOrNotSequenceEnd) -> Self {
            Self::BatchNotSequencedOrNotSequenceEnd(value)
        }
    }
    impl ::core::convert::From<ExceedMaxVerifyBatches> for AggchainBaseErrors {
        fn from(value: ExceedMaxVerifyBatches) -> Self {
            Self::ExceedMaxVerifyBatches(value)
        }
    }
    impl ::core::convert::From<FinalAccInputHashDoesNotMatch> for AggchainBaseErrors {
        fn from(value: FinalAccInputHashDoesNotMatch) -> Self {
            Self::FinalAccInputHashDoesNotMatch(value)
        }
    }
    impl ::core::convert::From<FinalNumBatchBelowLastVerifiedBatch>
    for AggchainBaseErrors {
        fn from(value: FinalNumBatchBelowLastVerifiedBatch) -> Self {
            Self::FinalNumBatchBelowLastVerifiedBatch(value)
        }
    }
    impl ::core::convert::From<FinalNumBatchDoesNotMatchPendingState>
    for AggchainBaseErrors {
        fn from(value: FinalNumBatchDoesNotMatchPendingState) -> Self {
            Self::FinalNumBatchDoesNotMatchPendingState(value)
        }
    }
    impl ::core::convert::From<FinalPendingStateNumInvalid> for AggchainBaseErrors {
        fn from(value: FinalPendingStateNumInvalid) -> Self {
            Self::FinalPendingStateNumInvalid(value)
        }
    }
    impl ::core::convert::From<ForceBatchNotAllowed> for AggchainBaseErrors {
        fn from(value: ForceBatchNotAllowed) -> Self {
            Self::ForceBatchNotAllowed(value)
        }
    }
    impl ::core::convert::From<ForceBatchTimeoutNotExpired> for AggchainBaseErrors {
        fn from(value: ForceBatchTimeoutNotExpired) -> Self {
            Self::ForceBatchTimeoutNotExpired(value)
        }
    }
    impl ::core::convert::From<ForceBatchesAlreadyActive> for AggchainBaseErrors {
        fn from(value: ForceBatchesAlreadyActive) -> Self {
            Self::ForceBatchesAlreadyActive(value)
        }
    }
    impl ::core::convert::From<ForceBatchesDecentralized> for AggchainBaseErrors {
        fn from(value: ForceBatchesDecentralized) -> Self {
            Self::ForceBatchesDecentralized(value)
        }
    }
    impl ::core::convert::From<ForceBatchesNotAllowedOnEmergencyState>
    for AggchainBaseErrors {
        fn from(value: ForceBatchesNotAllowedOnEmergencyState) -> Self {
            Self::ForceBatchesNotAllowedOnEmergencyState(value)
        }
    }
    impl ::core::convert::From<ForceBatchesOverflow> for AggchainBaseErrors {
        fn from(value: ForceBatchesOverflow) -> Self {
            Self::ForceBatchesOverflow(value)
        }
    }
    impl ::core::convert::From<ForcedDataDoesNotMatch> for AggchainBaseErrors {
        fn from(value: ForcedDataDoesNotMatch) -> Self {
            Self::ForcedDataDoesNotMatch(value)
        }
    }
    impl ::core::convert::From<GasTokenNetworkMustBeZeroOnEther> for AggchainBaseErrors {
        fn from(value: GasTokenNetworkMustBeZeroOnEther) -> Self {
            Self::GasTokenNetworkMustBeZeroOnEther(value)
        }
    }
    impl ::core::convert::From<GlobalExitRootNotExist> for AggchainBaseErrors {
        fn from(value: GlobalExitRootNotExist) -> Self {
            Self::GlobalExitRootNotExist(value)
        }
    }
    impl ::core::convert::From<HaltTimeoutNotExpired> for AggchainBaseErrors {
        fn from(value: HaltTimeoutNotExpired) -> Self {
            Self::HaltTimeoutNotExpired(value)
        }
    }
    impl ::core::convert::From<HaltTimeoutNotExpiredAfterEmergencyState>
    for AggchainBaseErrors {
        fn from(value: HaltTimeoutNotExpiredAfterEmergencyState) -> Self {
            Self::HaltTimeoutNotExpiredAfterEmergencyState(value)
        }
    }
    impl ::core::convert::From<HugeTokenMetadataNotSupported> for AggchainBaseErrors {
        fn from(value: HugeTokenMetadataNotSupported) -> Self {
            Self::HugeTokenMetadataNotSupported(value)
        }
    }
    impl ::core::convert::From<InitNumBatchAboveLastVerifiedBatch>
    for AggchainBaseErrors {
        fn from(value: InitNumBatchAboveLastVerifiedBatch) -> Self {
            Self::InitNumBatchAboveLastVerifiedBatch(value)
        }
    }
    impl ::core::convert::From<InitNumBatchDoesNotMatchPendingState>
    for AggchainBaseErrors {
        fn from(value: InitNumBatchDoesNotMatchPendingState) -> Self {
            Self::InitNumBatchDoesNotMatchPendingState(value)
        }
    }
    impl ::core::convert::From<InitSequencedBatchDoesNotMatch> for AggchainBaseErrors {
        fn from(value: InitSequencedBatchDoesNotMatch) -> Self {
            Self::InitSequencedBatchDoesNotMatch(value)
        }
    }
    impl ::core::convert::From<InvalidAggchainVKey> for AggchainBaseErrors {
        fn from(value: InvalidAggchainVKey) -> Self {
            Self::InvalidAggchainVKey(value)
        }
    }
    impl ::core::convert::From<InvalidInitializeFunction> for AggchainBaseErrors {
        fn from(value: InvalidInitializeFunction) -> Self {
            Self::InvalidInitializeFunction(value)
        }
    }
    impl ::core::convert::From<InvalidInitializeTransaction> for AggchainBaseErrors {
        fn from(value: InvalidInitializeTransaction) -> Self {
            Self::InvalidInitializeTransaction(value)
        }
    }
    impl ::core::convert::From<InvalidInitializer> for AggchainBaseErrors {
        fn from(value: InvalidInitializer) -> Self {
            Self::InvalidInitializer(value)
        }
    }
    impl ::core::convert::From<InvalidProof> for AggchainBaseErrors {
        fn from(value: InvalidProof) -> Self {
            Self::InvalidProof(value)
        }
    }
    impl ::core::convert::From<InvalidRangeBatchTimeTarget> for AggchainBaseErrors {
        fn from(value: InvalidRangeBatchTimeTarget) -> Self {
            Self::InvalidRangeBatchTimeTarget(value)
        }
    }
    impl ::core::convert::From<InvalidRangeForceBatchTimeout> for AggchainBaseErrors {
        fn from(value: InvalidRangeForceBatchTimeout) -> Self {
            Self::InvalidRangeForceBatchTimeout(value)
        }
    }
    impl ::core::convert::From<InvalidRangeMultiplierBatchFee> for AggchainBaseErrors {
        fn from(value: InvalidRangeMultiplierBatchFee) -> Self {
            Self::InvalidRangeMultiplierBatchFee(value)
        }
    }
    impl ::core::convert::From<L1InfoTreeLeafCountInvalid> for AggchainBaseErrors {
        fn from(value: L1InfoTreeLeafCountInvalid) -> Self {
            Self::L1InfoTreeLeafCountInvalid(value)
        }
    }
    impl ::core::convert::From<MaxTimestampSequenceInvalid> for AggchainBaseErrors {
        fn from(value: MaxTimestampSequenceInvalid) -> Self {
            Self::MaxTimestampSequenceInvalid(value)
        }
    }
    impl ::core::convert::From<NewAccInputHashDoesNotExist> for AggchainBaseErrors {
        fn from(value: NewAccInputHashDoesNotExist) -> Self {
            Self::NewAccInputHashDoesNotExist(value)
        }
    }
    impl ::core::convert::From<NewPendingStateTimeoutMustBeLower>
    for AggchainBaseErrors {
        fn from(value: NewPendingStateTimeoutMustBeLower) -> Self {
            Self::NewPendingStateTimeoutMustBeLower(value)
        }
    }
    impl ::core::convert::From<NewStateRootNotInsidePrime> for AggchainBaseErrors {
        fn from(value: NewStateRootNotInsidePrime) -> Self {
            Self::NewStateRootNotInsidePrime(value)
        }
    }
    impl ::core::convert::From<NewTrustedAggregatorTimeoutMustBeLower>
    for AggchainBaseErrors {
        fn from(value: NewTrustedAggregatorTimeoutMustBeLower) -> Self {
            Self::NewTrustedAggregatorTimeoutMustBeLower(value)
        }
    }
    impl ::core::convert::From<NotEnoughMaticAmount> for AggchainBaseErrors {
        fn from(value: NotEnoughMaticAmount) -> Self {
            Self::NotEnoughMaticAmount(value)
        }
    }
    impl ::core::convert::From<NotEnoughPOLAmount> for AggchainBaseErrors {
        fn from(value: NotEnoughPOLAmount) -> Self {
            Self::NotEnoughPOLAmount(value)
        }
    }
    impl ::core::convert::From<OldAccInputHashDoesNotExist> for AggchainBaseErrors {
        fn from(value: OldAccInputHashDoesNotExist) -> Self {
            Self::OldAccInputHashDoesNotExist(value)
        }
    }
    impl ::core::convert::From<OldStateRootDoesNotExist> for AggchainBaseErrors {
        fn from(value: OldStateRootDoesNotExist) -> Self {
            Self::OldStateRootDoesNotExist(value)
        }
    }
    impl ::core::convert::From<OnlyAdmin> for AggchainBaseErrors {
        fn from(value: OnlyAdmin) -> Self {
            Self::OnlyAdmin(value)
        }
    }
    impl ::core::convert::From<OnlyPendingAdmin> for AggchainBaseErrors {
        fn from(value: OnlyPendingAdmin) -> Self {
            Self::OnlyPendingAdmin(value)
        }
    }
    impl ::core::convert::From<OnlyPendingVKeyManager> for AggchainBaseErrors {
        fn from(value: OnlyPendingVKeyManager) -> Self {
            Self::OnlyPendingVKeyManager(value)
        }
    }
    impl ::core::convert::From<OnlyRollupManager> for AggchainBaseErrors {
        fn from(value: OnlyRollupManager) -> Self {
            Self::OnlyRollupManager(value)
        }
    }
    impl ::core::convert::From<OnlyTrustedAggregator> for AggchainBaseErrors {
        fn from(value: OnlyTrustedAggregator) -> Self {
            Self::OnlyTrustedAggregator(value)
        }
    }
    impl ::core::convert::From<OnlyTrustedSequencer> for AggchainBaseErrors {
        fn from(value: OnlyTrustedSequencer) -> Self {
            Self::OnlyTrustedSequencer(value)
        }
    }
    impl ::core::convert::From<OnlyVKeyManager> for AggchainBaseErrors {
        fn from(value: OnlyVKeyManager) -> Self {
            Self::OnlyVKeyManager(value)
        }
    }
    impl ::core::convert::From<OwnedAggchainVKeyAlreadyAdded> for AggchainBaseErrors {
        fn from(value: OwnedAggchainVKeyAlreadyAdded) -> Self {
            Self::OwnedAggchainVKeyAlreadyAdded(value)
        }
    }
    impl ::core::convert::From<OwnedAggchainVKeyLengthMismatch> for AggchainBaseErrors {
        fn from(value: OwnedAggchainVKeyLengthMismatch) -> Self {
            Self::OwnedAggchainVKeyLengthMismatch(value)
        }
    }
    impl ::core::convert::From<OwnedAggchainVKeyNotFound> for AggchainBaseErrors {
        fn from(value: OwnedAggchainVKeyNotFound) -> Self {
            Self::OwnedAggchainVKeyNotFound(value)
        }
    }
    impl ::core::convert::From<PendingStateDoesNotExist> for AggchainBaseErrors {
        fn from(value: PendingStateDoesNotExist) -> Self {
            Self::PendingStateDoesNotExist(value)
        }
    }
    impl ::core::convert::From<PendingStateInvalid> for AggchainBaseErrors {
        fn from(value: PendingStateInvalid) -> Self {
            Self::PendingStateInvalid(value)
        }
    }
    impl ::core::convert::From<PendingStateNotConsolidable> for AggchainBaseErrors {
        fn from(value: PendingStateNotConsolidable) -> Self {
            Self::PendingStateNotConsolidable(value)
        }
    }
    impl ::core::convert::From<PendingStateTimeoutExceedHaltAggregationTimeout>
    for AggchainBaseErrors {
        fn from(value: PendingStateTimeoutExceedHaltAggregationTimeout) -> Self {
            Self::PendingStateTimeoutExceedHaltAggregationTimeout(value)
        }
    }
    impl ::core::convert::From<SequenceZeroBatches> for AggchainBaseErrors {
        fn from(value: SequenceZeroBatches) -> Self {
            Self::SequenceZeroBatches(value)
        }
    }
    impl ::core::convert::From<SequencedTimestampBelowForcedTimestamp>
    for AggchainBaseErrors {
        fn from(value: SequencedTimestampBelowForcedTimestamp) -> Self {
            Self::SequencedTimestampBelowForcedTimestamp(value)
        }
    }
    impl ::core::convert::From<SequencedTimestampInvalid> for AggchainBaseErrors {
        fn from(value: SequencedTimestampInvalid) -> Self {
            Self::SequencedTimestampInvalid(value)
        }
    }
    impl ::core::convert::From<StoredRootMustBeDifferentThanNewRoot>
    for AggchainBaseErrors {
        fn from(value: StoredRootMustBeDifferentThanNewRoot) -> Self {
            Self::StoredRootMustBeDifferentThanNewRoot(value)
        }
    }
    impl ::core::convert::From<TransactionsLengthAboveMax> for AggchainBaseErrors {
        fn from(value: TransactionsLengthAboveMax) -> Self {
            Self::TransactionsLengthAboveMax(value)
        }
    }
    impl ::core::convert::From<TrustedAggregatorTimeoutExceedHaltAggregationTimeout>
    for AggchainBaseErrors {
        fn from(value: TrustedAggregatorTimeoutExceedHaltAggregationTimeout) -> Self {
            Self::TrustedAggregatorTimeoutExceedHaltAggregationTimeout(value)
        }
    }
    impl ::core::convert::From<TrustedAggregatorTimeoutNotExpired>
    for AggchainBaseErrors {
        fn from(value: TrustedAggregatorTimeoutNotExpired) -> Self {
            Self::TrustedAggregatorTimeoutNotExpired(value)
        }
    }
    impl ::core::convert::From<UseDefaultGatewayAlreadySet> for AggchainBaseErrors {
        fn from(value: UseDefaultGatewayAlreadySet) -> Self {
            Self::UseDefaultGatewayAlreadySet(value)
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
    #[ethevent(name = "AcceptVKeyManagerRole", abi = "AcceptVKeyManagerRole(address)")]
    pub struct AcceptVKeyManagerRoleFilter {
        pub new_v_key_manager: ::ethers::core::types::Address,
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
    #[ethevent(name = "AddAggchainVKey", abi = "AddAggchainVKey(bytes4,bytes32)")]
    pub struct AddAggchainVKeyFilter {
        pub selector: [u8; 4],
        pub new_aggchain_v_key: [u8; 32],
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
    #[ethevent(
        name = "TransferVKeyManagerRole",
        abi = "TransferVKeyManagerRole(address)"
    )]
    pub struct TransferVKeyManagerRoleFilter {
        pub new_v_key_manager: ::ethers::core::types::Address,
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
    #[ethevent(name = "UpdateAggchainVKey", abi = "UpdateAggchainVKey(bytes4,bytes32)")]
    pub struct UpdateAggchainVKeyFilter {
        pub selector: [u8; 4],
        pub new_aggchain_v_key: [u8; 32],
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
        name = "UpdateUseDefaultGatewayFlag",
        abi = "UpdateUseDefaultGatewayFlag(bool)"
    )]
    pub struct UpdateUseDefaultGatewayFlagFilter {
        pub use_default_gateway: bool,
    }
    ///Container type for all of the contract's events
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum AggchainBaseEvents {
        AcceptAdminRoleFilter(AcceptAdminRoleFilter),
        AcceptVKeyManagerRoleFilter(AcceptVKeyManagerRoleFilter),
        AddAggchainVKeyFilter(AddAggchainVKeyFilter),
        InitializedFilter(InitializedFilter),
        SetTrustedSequencerFilter(SetTrustedSequencerFilter),
        SetTrustedSequencerURLFilter(SetTrustedSequencerURLFilter),
        TransferAdminRoleFilter(TransferAdminRoleFilter),
        TransferVKeyManagerRoleFilter(TransferVKeyManagerRoleFilter),
        UpdateAggchainVKeyFilter(UpdateAggchainVKeyFilter),
        UpdateUseDefaultGatewayFlagFilter(UpdateUseDefaultGatewayFlagFilter),
    }
    impl ::ethers::contract::EthLogDecode for AggchainBaseEvents {
        fn decode_log(
            log: &::ethers::core::abi::RawLog,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::Error> {
            if let Ok(decoded) = AcceptAdminRoleFilter::decode_log(log) {
                return Ok(AggchainBaseEvents::AcceptAdminRoleFilter(decoded));
            }
            if let Ok(decoded) = AcceptVKeyManagerRoleFilter::decode_log(log) {
                return Ok(AggchainBaseEvents::AcceptVKeyManagerRoleFilter(decoded));
            }
            if let Ok(decoded) = AddAggchainVKeyFilter::decode_log(log) {
                return Ok(AggchainBaseEvents::AddAggchainVKeyFilter(decoded));
            }
            if let Ok(decoded) = InitializedFilter::decode_log(log) {
                return Ok(AggchainBaseEvents::InitializedFilter(decoded));
            }
            if let Ok(decoded) = SetTrustedSequencerFilter::decode_log(log) {
                return Ok(AggchainBaseEvents::SetTrustedSequencerFilter(decoded));
            }
            if let Ok(decoded) = SetTrustedSequencerURLFilter::decode_log(log) {
                return Ok(AggchainBaseEvents::SetTrustedSequencerURLFilter(decoded));
            }
            if let Ok(decoded) = TransferAdminRoleFilter::decode_log(log) {
                return Ok(AggchainBaseEvents::TransferAdminRoleFilter(decoded));
            }
            if let Ok(decoded) = TransferVKeyManagerRoleFilter::decode_log(log) {
                return Ok(AggchainBaseEvents::TransferVKeyManagerRoleFilter(decoded));
            }
            if let Ok(decoded) = UpdateAggchainVKeyFilter::decode_log(log) {
                return Ok(AggchainBaseEvents::UpdateAggchainVKeyFilter(decoded));
            }
            if let Ok(decoded) = UpdateUseDefaultGatewayFlagFilter::decode_log(log) {
                return Ok(
                    AggchainBaseEvents::UpdateUseDefaultGatewayFlagFilter(decoded),
                );
            }
            Err(::ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::core::fmt::Display for AggchainBaseEvents {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::AcceptAdminRoleFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::AcceptVKeyManagerRoleFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::AddAggchainVKeyFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InitializedFilter(element) => ::core::fmt::Display::fmt(element, f),
                Self::SetTrustedSequencerFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::SetTrustedSequencerURLFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::TransferAdminRoleFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::TransferVKeyManagerRoleFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::UpdateAggchainVKeyFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::UpdateUseDefaultGatewayFlagFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
            }
        }
    }
    impl ::core::convert::From<AcceptAdminRoleFilter> for AggchainBaseEvents {
        fn from(value: AcceptAdminRoleFilter) -> Self {
            Self::AcceptAdminRoleFilter(value)
        }
    }
    impl ::core::convert::From<AcceptVKeyManagerRoleFilter> for AggchainBaseEvents {
        fn from(value: AcceptVKeyManagerRoleFilter) -> Self {
            Self::AcceptVKeyManagerRoleFilter(value)
        }
    }
    impl ::core::convert::From<AddAggchainVKeyFilter> for AggchainBaseEvents {
        fn from(value: AddAggchainVKeyFilter) -> Self {
            Self::AddAggchainVKeyFilter(value)
        }
    }
    impl ::core::convert::From<InitializedFilter> for AggchainBaseEvents {
        fn from(value: InitializedFilter) -> Self {
            Self::InitializedFilter(value)
        }
    }
    impl ::core::convert::From<SetTrustedSequencerFilter> for AggchainBaseEvents {
        fn from(value: SetTrustedSequencerFilter) -> Self {
            Self::SetTrustedSequencerFilter(value)
        }
    }
    impl ::core::convert::From<SetTrustedSequencerURLFilter> for AggchainBaseEvents {
        fn from(value: SetTrustedSequencerURLFilter) -> Self {
            Self::SetTrustedSequencerURLFilter(value)
        }
    }
    impl ::core::convert::From<TransferAdminRoleFilter> for AggchainBaseEvents {
        fn from(value: TransferAdminRoleFilter) -> Self {
            Self::TransferAdminRoleFilter(value)
        }
    }
    impl ::core::convert::From<TransferVKeyManagerRoleFilter> for AggchainBaseEvents {
        fn from(value: TransferVKeyManagerRoleFilter) -> Self {
            Self::TransferVKeyManagerRoleFilter(value)
        }
    }
    impl ::core::convert::From<UpdateAggchainVKeyFilter> for AggchainBaseEvents {
        fn from(value: UpdateAggchainVKeyFilter) -> Self {
            Self::UpdateAggchainVKeyFilter(value)
        }
    }
    impl ::core::convert::From<UpdateUseDefaultGatewayFlagFilter>
    for AggchainBaseEvents {
        fn from(value: UpdateUseDefaultGatewayFlagFilter) -> Self {
            Self::UpdateUseDefaultGatewayFlagFilter(value)
        }
    }
    ///Container type for all input parameters for the `AGGCHAIN_TYPE` function with signature `AGGCHAIN_TYPE()` and selector `0x6e7fbce9`
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
    #[ethcall(name = "AGGCHAIN_TYPE", abi = "AGGCHAIN_TYPE()")]
    pub struct AggchainTypeCall;
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
    ///Container type for all input parameters for the `acceptVKeyManagerRole` function with signature `acceptVKeyManagerRole()` and selector `0x368c822c`
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
    #[ethcall(name = "acceptVKeyManagerRole", abi = "acceptVKeyManagerRole()")]
    pub struct AcceptVKeyManagerRoleCall;
    ///Container type for all input parameters for the `addOwnedAggchainVKey` function with signature `addOwnedAggchainVKey(bytes4,bytes32)` and selector `0x19451a8f`
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
        name = "addOwnedAggchainVKey",
        abi = "addOwnedAggchainVKey(bytes4,bytes32)"
    )]
    pub struct AddOwnedAggchainVKeyCall {
        pub aggchain_selector: [u8; 4],
        pub new_aggchain_v_key: [u8; 32],
    }
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
    ///Container type for all input parameters for the `disableUseDefaultGatewayFlag` function with signature `disableUseDefaultGatewayFlag()` and selector `0xdc8c4249`
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
        name = "disableUseDefaultGatewayFlag",
        abi = "disableUseDefaultGatewayFlag()"
    )]
    pub struct DisableUseDefaultGatewayFlagCall;
    ///Container type for all input parameters for the `enableUseDefaultGatewayFlag` function with signature `enableUseDefaultGatewayFlag()` and selector `0xe631476c`
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
        name = "enableUseDefaultGatewayFlag",
        abi = "enableUseDefaultGatewayFlag()"
    )]
    pub struct EnableUseDefaultGatewayFlagCall;
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
    ///Container type for all input parameters for the `getAggchainVKey` function with signature `getAggchainVKey(bytes4)` and selector `0x01fcf6a0`
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
    #[ethcall(name = "getAggchainVKey", abi = "getAggchainVKey(bytes4)")]
    pub struct GetAggchainVKeyCall {
        pub aggchain_selector: [u8; 4],
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
    pub struct InitializeCall(
        pub ::ethers::core::types::Address,
        pub ::ethers::core::types::Address,
        pub u32,
        pub ::ethers::core::types::Address,
        pub ::std::string::String,
        pub ::std::string::String,
    );
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
    ///Container type for all input parameters for the `ownedAggchainVKeys` function with signature `ownedAggchainVKeys(bytes4)` and selector `0xeffb8479`
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
    #[ethcall(name = "ownedAggchainVKeys", abi = "ownedAggchainVKeys(bytes4)")]
    pub struct OwnedAggchainVKeysCall {
        pub aggchain_v_key_selector: [u8; 4],
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
    ///Container type for all input parameters for the `pendingVKeyManager` function with signature `pendingVKeyManager()` and selector `0xbfb193b6`
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
    #[ethcall(name = "pendingVKeyManager", abi = "pendingVKeyManager()")]
    pub struct PendingVKeyManagerCall;
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
    ///Container type for all input parameters for the `transferVKeyManagerRole` function with signature `transferVKeyManagerRole(address)` and selector `0x85018182`
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
        name = "transferVKeyManagerRole",
        abi = "transferVKeyManagerRole(address)"
    )]
    pub struct TransferVKeyManagerRoleCall {
        pub new_v_key_manager: ::ethers::core::types::Address,
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
    ///Container type for all input parameters for the `updateOwnedAggchainVKey` function with signature `updateOwnedAggchainVKey(bytes4,bytes32)` and selector `0x314eb17b`
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
        name = "updateOwnedAggchainVKey",
        abi = "updateOwnedAggchainVKey(bytes4,bytes32)"
    )]
    pub struct UpdateOwnedAggchainVKeyCall {
        pub aggchain_selector: [u8; 4],
        pub updated_aggchain_v_key: [u8; 32],
    }
    ///Container type for all input parameters for the `useDefaultGateway` function with signature `useDefaultGateway()` and selector `0xff904079`
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
    #[ethcall(name = "useDefaultGateway", abi = "useDefaultGateway()")]
    pub struct UseDefaultGatewayCall;
    ///Container type for all input parameters for the `vKeyManager` function with signature `vKeyManager()` and selector `0xe279984e`
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
    #[ethcall(name = "vKeyManager", abi = "vKeyManager()")]
    pub struct VkeyManagerCall;
    ///Container type for all of the contract's call
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum AggchainBaseCalls {
        AggchainType(AggchainTypeCall),
        AcceptAdminRole(AcceptAdminRoleCall),
        AcceptVKeyManagerRole(AcceptVKeyManagerRoleCall),
        AddOwnedAggchainVKey(AddOwnedAggchainVKeyCall),
        Admin(AdminCall),
        AggLayerGateway(AggLayerGatewayCall),
        BridgeAddress(BridgeAddressCall),
        DisableUseDefaultGatewayFlag(DisableUseDefaultGatewayFlagCall),
        EnableUseDefaultGatewayFlag(EnableUseDefaultGatewayFlagCall),
        ForceBatchAddress(ForceBatchAddressCall),
        ForceBatchTimeout(ForceBatchTimeoutCall),
        ForcedBatches(ForcedBatchesCall),
        GasTokenAddress(GasTokenAddressCall),
        GasTokenNetwork(GasTokenNetworkCall),
        GetAggchainVKey(GetAggchainVKeyCall),
        GlobalExitRootManager(GlobalExitRootManagerCall),
        Initialize(InitializeCall),
        LastAccInputHash(LastAccInputHashCall),
        LastForceBatch(LastForceBatchCall),
        LastForceBatchSequenced(LastForceBatchSequencedCall),
        NetworkName(NetworkNameCall),
        OwnedAggchainVKeys(OwnedAggchainVKeysCall),
        PendingAdmin(PendingAdminCall),
        PendingVKeyManager(PendingVKeyManagerCall),
        Pol(PolCall),
        RollupManager(RollupManagerCall),
        SetTrustedSequencer(SetTrustedSequencerCall),
        SetTrustedSequencerURL(SetTrustedSequencerURLCall),
        TransferAdminRole(TransferAdminRoleCall),
        TransferVKeyManagerRole(TransferVKeyManagerRoleCall),
        TrustedSequencer(TrustedSequencerCall),
        TrustedSequencerURL(TrustedSequencerURLCall),
        UpdateOwnedAggchainVKey(UpdateOwnedAggchainVKeyCall),
        UseDefaultGateway(UseDefaultGatewayCall),
        VkeyManager(VkeyManagerCall),
    }
    impl ::ethers::core::abi::AbiDecode for AggchainBaseCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded) = <AggchainTypeCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::AggchainType(decoded));
            }
            if let Ok(decoded) = <AcceptAdminRoleCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::AcceptAdminRole(decoded));
            }
            if let Ok(decoded) = <AcceptVKeyManagerRoleCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::AcceptVKeyManagerRole(decoded));
            }
            if let Ok(decoded) = <AddOwnedAggchainVKeyCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::AddOwnedAggchainVKey(decoded));
            }
            if let Ok(decoded) = <AdminCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::Admin(decoded));
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
            if let Ok(decoded) = <DisableUseDefaultGatewayFlagCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::DisableUseDefaultGatewayFlag(decoded));
            }
            if let Ok(decoded) = <EnableUseDefaultGatewayFlagCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::EnableUseDefaultGatewayFlag(decoded));
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
            if let Ok(decoded) = <GetAggchainVKeyCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GetAggchainVKey(decoded));
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
            if let Ok(decoded) = <OwnedAggchainVKeysCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::OwnedAggchainVKeys(decoded));
            }
            if let Ok(decoded) = <PendingAdminCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::PendingAdmin(decoded));
            }
            if let Ok(decoded) = <PendingVKeyManagerCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::PendingVKeyManager(decoded));
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
            if let Ok(decoded) = <TransferVKeyManagerRoleCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::TransferVKeyManagerRole(decoded));
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
            if let Ok(decoded) = <UpdateOwnedAggchainVKeyCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::UpdateOwnedAggchainVKey(decoded));
            }
            if let Ok(decoded) = <UseDefaultGatewayCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::UseDefaultGateway(decoded));
            }
            if let Ok(decoded) = <VkeyManagerCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::VkeyManager(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for AggchainBaseCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                Self::AggchainType(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::AcceptAdminRole(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::AcceptVKeyManagerRole(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::AddOwnedAggchainVKey(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::Admin(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::AggLayerGateway(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::BridgeAddress(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::DisableUseDefaultGatewayFlag(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::EnableUseDefaultGatewayFlag(element) => {
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
                Self::GetAggchainVKey(element) => {
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
                Self::OwnedAggchainVKeys(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::PendingAdmin(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::PendingVKeyManager(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::Pol(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::RollupManager(element) => {
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
                Self::TransferVKeyManagerRole(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::TrustedSequencer(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::TrustedSequencerURL(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::UpdateOwnedAggchainVKey(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::UseDefaultGateway(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::VkeyManager(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
            }
        }
    }
    impl ::core::fmt::Display for AggchainBaseCalls {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::AggchainType(element) => ::core::fmt::Display::fmt(element, f),
                Self::AcceptAdminRole(element) => ::core::fmt::Display::fmt(element, f),
                Self::AcceptVKeyManagerRole(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::AddOwnedAggchainVKey(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::Admin(element) => ::core::fmt::Display::fmt(element, f),
                Self::AggLayerGateway(element) => ::core::fmt::Display::fmt(element, f),
                Self::BridgeAddress(element) => ::core::fmt::Display::fmt(element, f),
                Self::DisableUseDefaultGatewayFlag(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::EnableUseDefaultGatewayFlag(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ForceBatchAddress(element) => ::core::fmt::Display::fmt(element, f),
                Self::ForceBatchTimeout(element) => ::core::fmt::Display::fmt(element, f),
                Self::ForcedBatches(element) => ::core::fmt::Display::fmt(element, f),
                Self::GasTokenAddress(element) => ::core::fmt::Display::fmt(element, f),
                Self::GasTokenNetwork(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetAggchainVKey(element) => ::core::fmt::Display::fmt(element, f),
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
                Self::OwnedAggchainVKeys(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::PendingAdmin(element) => ::core::fmt::Display::fmt(element, f),
                Self::PendingVKeyManager(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::Pol(element) => ::core::fmt::Display::fmt(element, f),
                Self::RollupManager(element) => ::core::fmt::Display::fmt(element, f),
                Self::SetTrustedSequencer(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::SetTrustedSequencerURL(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::TransferAdminRole(element) => ::core::fmt::Display::fmt(element, f),
                Self::TransferVKeyManagerRole(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::TrustedSequencer(element) => ::core::fmt::Display::fmt(element, f),
                Self::TrustedSequencerURL(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::UpdateOwnedAggchainVKey(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::UseDefaultGateway(element) => ::core::fmt::Display::fmt(element, f),
                Self::VkeyManager(element) => ::core::fmt::Display::fmt(element, f),
            }
        }
    }
    impl ::core::convert::From<AggchainTypeCall> for AggchainBaseCalls {
        fn from(value: AggchainTypeCall) -> Self {
            Self::AggchainType(value)
        }
    }
    impl ::core::convert::From<AcceptAdminRoleCall> for AggchainBaseCalls {
        fn from(value: AcceptAdminRoleCall) -> Self {
            Self::AcceptAdminRole(value)
        }
    }
    impl ::core::convert::From<AcceptVKeyManagerRoleCall> for AggchainBaseCalls {
        fn from(value: AcceptVKeyManagerRoleCall) -> Self {
            Self::AcceptVKeyManagerRole(value)
        }
    }
    impl ::core::convert::From<AddOwnedAggchainVKeyCall> for AggchainBaseCalls {
        fn from(value: AddOwnedAggchainVKeyCall) -> Self {
            Self::AddOwnedAggchainVKey(value)
        }
    }
    impl ::core::convert::From<AdminCall> for AggchainBaseCalls {
        fn from(value: AdminCall) -> Self {
            Self::Admin(value)
        }
    }
    impl ::core::convert::From<AggLayerGatewayCall> for AggchainBaseCalls {
        fn from(value: AggLayerGatewayCall) -> Self {
            Self::AggLayerGateway(value)
        }
    }
    impl ::core::convert::From<BridgeAddressCall> for AggchainBaseCalls {
        fn from(value: BridgeAddressCall) -> Self {
            Self::BridgeAddress(value)
        }
    }
    impl ::core::convert::From<DisableUseDefaultGatewayFlagCall> for AggchainBaseCalls {
        fn from(value: DisableUseDefaultGatewayFlagCall) -> Self {
            Self::DisableUseDefaultGatewayFlag(value)
        }
    }
    impl ::core::convert::From<EnableUseDefaultGatewayFlagCall> for AggchainBaseCalls {
        fn from(value: EnableUseDefaultGatewayFlagCall) -> Self {
            Self::EnableUseDefaultGatewayFlag(value)
        }
    }
    impl ::core::convert::From<ForceBatchAddressCall> for AggchainBaseCalls {
        fn from(value: ForceBatchAddressCall) -> Self {
            Self::ForceBatchAddress(value)
        }
    }
    impl ::core::convert::From<ForceBatchTimeoutCall> for AggchainBaseCalls {
        fn from(value: ForceBatchTimeoutCall) -> Self {
            Self::ForceBatchTimeout(value)
        }
    }
    impl ::core::convert::From<ForcedBatchesCall> for AggchainBaseCalls {
        fn from(value: ForcedBatchesCall) -> Self {
            Self::ForcedBatches(value)
        }
    }
    impl ::core::convert::From<GasTokenAddressCall> for AggchainBaseCalls {
        fn from(value: GasTokenAddressCall) -> Self {
            Self::GasTokenAddress(value)
        }
    }
    impl ::core::convert::From<GasTokenNetworkCall> for AggchainBaseCalls {
        fn from(value: GasTokenNetworkCall) -> Self {
            Self::GasTokenNetwork(value)
        }
    }
    impl ::core::convert::From<GetAggchainVKeyCall> for AggchainBaseCalls {
        fn from(value: GetAggchainVKeyCall) -> Self {
            Self::GetAggchainVKey(value)
        }
    }
    impl ::core::convert::From<GlobalExitRootManagerCall> for AggchainBaseCalls {
        fn from(value: GlobalExitRootManagerCall) -> Self {
            Self::GlobalExitRootManager(value)
        }
    }
    impl ::core::convert::From<InitializeCall> for AggchainBaseCalls {
        fn from(value: InitializeCall) -> Self {
            Self::Initialize(value)
        }
    }
    impl ::core::convert::From<LastAccInputHashCall> for AggchainBaseCalls {
        fn from(value: LastAccInputHashCall) -> Self {
            Self::LastAccInputHash(value)
        }
    }
    impl ::core::convert::From<LastForceBatchCall> for AggchainBaseCalls {
        fn from(value: LastForceBatchCall) -> Self {
            Self::LastForceBatch(value)
        }
    }
    impl ::core::convert::From<LastForceBatchSequencedCall> for AggchainBaseCalls {
        fn from(value: LastForceBatchSequencedCall) -> Self {
            Self::LastForceBatchSequenced(value)
        }
    }
    impl ::core::convert::From<NetworkNameCall> for AggchainBaseCalls {
        fn from(value: NetworkNameCall) -> Self {
            Self::NetworkName(value)
        }
    }
    impl ::core::convert::From<OwnedAggchainVKeysCall> for AggchainBaseCalls {
        fn from(value: OwnedAggchainVKeysCall) -> Self {
            Self::OwnedAggchainVKeys(value)
        }
    }
    impl ::core::convert::From<PendingAdminCall> for AggchainBaseCalls {
        fn from(value: PendingAdminCall) -> Self {
            Self::PendingAdmin(value)
        }
    }
    impl ::core::convert::From<PendingVKeyManagerCall> for AggchainBaseCalls {
        fn from(value: PendingVKeyManagerCall) -> Self {
            Self::PendingVKeyManager(value)
        }
    }
    impl ::core::convert::From<PolCall> for AggchainBaseCalls {
        fn from(value: PolCall) -> Self {
            Self::Pol(value)
        }
    }
    impl ::core::convert::From<RollupManagerCall> for AggchainBaseCalls {
        fn from(value: RollupManagerCall) -> Self {
            Self::RollupManager(value)
        }
    }
    impl ::core::convert::From<SetTrustedSequencerCall> for AggchainBaseCalls {
        fn from(value: SetTrustedSequencerCall) -> Self {
            Self::SetTrustedSequencer(value)
        }
    }
    impl ::core::convert::From<SetTrustedSequencerURLCall> for AggchainBaseCalls {
        fn from(value: SetTrustedSequencerURLCall) -> Self {
            Self::SetTrustedSequencerURL(value)
        }
    }
    impl ::core::convert::From<TransferAdminRoleCall> for AggchainBaseCalls {
        fn from(value: TransferAdminRoleCall) -> Self {
            Self::TransferAdminRole(value)
        }
    }
    impl ::core::convert::From<TransferVKeyManagerRoleCall> for AggchainBaseCalls {
        fn from(value: TransferVKeyManagerRoleCall) -> Self {
            Self::TransferVKeyManagerRole(value)
        }
    }
    impl ::core::convert::From<TrustedSequencerCall> for AggchainBaseCalls {
        fn from(value: TrustedSequencerCall) -> Self {
            Self::TrustedSequencer(value)
        }
    }
    impl ::core::convert::From<TrustedSequencerURLCall> for AggchainBaseCalls {
        fn from(value: TrustedSequencerURLCall) -> Self {
            Self::TrustedSequencerURL(value)
        }
    }
    impl ::core::convert::From<UpdateOwnedAggchainVKeyCall> for AggchainBaseCalls {
        fn from(value: UpdateOwnedAggchainVKeyCall) -> Self {
            Self::UpdateOwnedAggchainVKey(value)
        }
    }
    impl ::core::convert::From<UseDefaultGatewayCall> for AggchainBaseCalls {
        fn from(value: UseDefaultGatewayCall) -> Self {
            Self::UseDefaultGateway(value)
        }
    }
    impl ::core::convert::From<VkeyManagerCall> for AggchainBaseCalls {
        fn from(value: VkeyManagerCall) -> Self {
            Self::VkeyManager(value)
        }
    }
    ///Container type for all return fields from the `AGGCHAIN_TYPE` function with signature `AGGCHAIN_TYPE()` and selector `0x6e7fbce9`
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
    pub struct AggchainTypeReturn(pub u32);
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
    ///Container type for all return fields from the `getAggchainVKey` function with signature `getAggchainVKey(bytes4)` and selector `0x01fcf6a0`
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
    pub struct GetAggchainVKeyReturn {
        pub aggchain_v_key: [u8; 32],
    }
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
    ///Container type for all return fields from the `ownedAggchainVKeys` function with signature `ownedAggchainVKeys(bytes4)` and selector `0xeffb8479`
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
    pub struct OwnedAggchainVKeysReturn {
        pub owned_aggchain_v_key: [u8; 32],
    }
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
    ///Container type for all return fields from the `pendingVKeyManager` function with signature `pendingVKeyManager()` and selector `0xbfb193b6`
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
    pub struct PendingVKeyManagerReturn(pub ::ethers::core::types::Address);
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
    ///Container type for all return fields from the `useDefaultGateway` function with signature `useDefaultGateway()` and selector `0xff904079`
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
    pub struct UseDefaultGatewayReturn(pub bool);
    ///Container type for all return fields from the `vKeyManager` function with signature `vKeyManager()` and selector `0xe279984e`
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
    pub struct VkeyManagerReturn(pub ::ethers::core::types::Address);
}
