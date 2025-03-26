pub use agg_layer_gateway::*;
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
pub mod agg_layer_gateway {
    #[allow(deprecated)]
    fn __abi() -> ::ethers::core::abi::Abi {
        ::ethers::core::abi::ethabi::Contract {
            constructor: ::core::option::Option::Some(::ethers::core::abi::ethabi::Constructor {
                inputs: ::std::vec![],
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
                    ::std::borrow::ToOwned::to_owned("addDefaultAggchainVKey"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "addDefaultAggchainVKey",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "defaultAggchainSelector",
                                    ),
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
                    ::std::borrow::ToOwned::to_owned("addPessimisticVKeyRoute"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "addPessimisticVKeyRoute",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "pessimisticVKeySelector",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        4usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes4"),
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
                                    name: ::std::borrow::ToOwned::to_owned("pessimisticVKey"),
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
                    ::std::borrow::ToOwned::to_owned("defaultAggchainVKeys"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "defaultAggchainVKeys",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "defaultAggchainSelector",
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
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "defaultAggchainVKey",
                                    ),
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
                    ::std::borrow::ToOwned::to_owned("freezePessimisticVKeyRoute"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "freezePessimisticVKeyRoute",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "pessimisticVKeySelector",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        4usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes4"),
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
                    ::std::borrow::ToOwned::to_owned("getDefaultAggchainVKey"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "getDefaultAggchainVKey",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "defaultAggchainSelector",
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
                                    name: ::std::borrow::ToOwned::to_owned("defaultAdmin"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "aggchainDefaultVKeyRole",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("addRouteRole"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("freezeRouteRole"),
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
                    ::std::borrow::ToOwned::to_owned("pessimisticVKeyRoutes"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "pessimisticVKeyRoutes",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "pessimisticVKeySelector",
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
                                    name: ::std::borrow::ToOwned::to_owned("verifier"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("pessimisticVKey"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("frozen"),
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
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "callerConfirmation",
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
                    ::std::borrow::ToOwned::to_owned("supportsInterface"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("supportsInterface"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("interfaceId"),
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
                    ::std::borrow::ToOwned::to_owned("updateDefaultAggchainVKey"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "updateDefaultAggchainVKey",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "defaultAggchainSelector",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        4usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes4"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "newDefaultAggchainVKey",
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
                    ::std::borrow::ToOwned::to_owned("verifyPessimisticProof"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "verifyPessimisticProof",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("publicValues"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("proofBytes"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
            ]),
            events: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("AcceptAggLayerAdminRole"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned(
                                "AcceptAggLayerAdminRole",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("newAggLayerAdmin"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("AddDefaultAggchainVKey"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned(
                                "AddDefaultAggchainVKey",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("selector"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        4usize,
                                    ),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("newVKey"),
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
                    ::std::borrow::ToOwned::to_owned("RouteAdded"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("RouteAdded"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("selector"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        4usize,
                                    ),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("verifier"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("pessimisticVKey"),
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
                    ::std::borrow::ToOwned::to_owned("RouteFrozen"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("RouteFrozen"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("selector"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        4usize,
                                    ),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("verifier"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("TransferAggLayerAdminRole"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned(
                                "TransferAggLayerAdminRole",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "newPendingAggLayerAdmin",
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
                    ::std::borrow::ToOwned::to_owned("UpdateDefaultAggchainVKey"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned(
                                "UpdateDefaultAggchainVKey",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("selector"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        4usize,
                                    ),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("newVKey"),
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
                    ::std::borrow::ToOwned::to_owned("UpdatePessimisticVKey"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned(
                                "UpdatePessimisticVKey",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("selector"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        4usize,
                                    ),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("verifier"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "newPessimisticVKey",
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
            ]),
            errors: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("AccessControlBadConfirmation"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "AccessControlBadConfirmation",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("AccessControlUnauthorizedAccount"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "AccessControlUnauthorizedAccount",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("account"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("neededRole"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("AggchainVKeyAlreadyExists"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "AggchainVKeyAlreadyExists",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
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
                    ::std::borrow::ToOwned::to_owned("OnlyAggLayerAdmin"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("OnlyAggLayerAdmin"),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("OnlyPendingAggLayerAdmin"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "OnlyPendingAggLayerAdmin",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("RouteAlreadyExists"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("RouteAlreadyExists"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("verifier"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("RouteIsFrozen"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("RouteIsFrozen"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("selector"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        4usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes4"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("RouteNotFound"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("RouteNotFound"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("selector"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        4usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes4"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("SelectorCannotBeZero"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "SelectorCannotBeZero",
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
    pub static AGGLAYERGATEWAY_ABI: ::ethers::contract::Lazy<::ethers::core::abi::Abi> = ::ethers::contract::Lazy::new(
        __abi,
    );
    #[rustfmt::skip]
    const __BYTECODE: &[u8] = b"`\x80`@R4\x80\x15`\x0EW__\xFD[P`\x15`\x19V[`\xD4V[_Ta\x01\0\x90\x04`\xFF\x16\x15`\x83W`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`'`$\x82\x01R\x7FInitializable: contract is initi`D\x82\x01Rfalizing`\xC8\x1B`d\x82\x01R`\x84\x01`@Q\x80\x91\x03\x90\xFD[_T`\xFF\x90\x81\x16\x10\x15`\xD2W_\x80T`\xFF\x19\x16`\xFF\x90\x81\x17\x90\x91U`@Q\x90\x81R\x7F\x7F&\xB8?\xF9n\x1F+jh/\x138R\xF6y\x8A\t\xC4e\xDA\x95\x92\x14`\xCE\xFB8G@$\x98\x90` \x01`@Q\x80\x91\x03\x90\xA1[V[a\x13\xDC\x80a\0\xE1_9_\xF3\xFE`\x80`@R4\x80\x15a\0\x0FW__\xFD[P`\x046\x10a\0\xFBW_5`\xE0\x1C\x80c\x91\xD1HT\x11a\0\x93W\x80c\xA4\x8F\xD3w\x11a\0cW\x80c\xA4\x8F\xD3w\x14a\x02\xB3W\x80c\xD5Gt\x1F\x14a\x02\xC6W\x80c\xF4\xC4F\x81\x14a\x02\xD9W\x80c\xF8\xC8v^\x14a\x02\xECW__\xFD[\x80c\x91\xD1HT\x14a\x02AW\x80c\x95\x84\xA5\x16\x14a\x02\x86W\x80c\x95\xD2\xA3\x91\x14a\x02\x99W\x80c\xA2\x17\xFD\xDF\x14a\x02\xACW__\xFD[\x80cl\xAB\xFD\xAB\x11a\0\xCEW\x80cl\xAB\xFD\xAB\x14a\x01\x80W\x80cqk\xE0u\x14a\x01\x93W\x80c\x81\x8C\x8C!\x14a\x02\x0FW\x80c\x82\xBF\xAE\xA1\x14a\x02\"W__\xFD[\x80c\x01\xFF\xC9\xA7\x14a\0\xFFW\x80c$\x8A\x9C\xA3\x14a\x01'W\x80c//\xF1]\x14a\x01XW\x80c6V\x8A\xBE\x14a\x01mW[__\xFD[a\x01\x12a\x01\r6`\x04a\x10\xB2V[a\x02\xFFV[`@Q\x90\x15\x15\x81R` \x01[`@Q\x80\x91\x03\x90\xF3[a\x01Ja\x0156`\x04a\x10\xD2V[_\x90\x81R`\x01` \x81\x90R`@\x90\x91 \x01T\x90V[`@Q\x90\x81R` \x01a\x01\x1EV[a\x01ka\x01f6`\x04a\x11\x0CV[a\x03\x97V[\0[a\x01ka\x01{6`\x04a\x11\x0CV[a\x03\xC2V[a\x01Ja\x01\x8E6`\x04a\x10\xB2V[a\x04 V[a\x01\xDBa\x01\xA16`\x04a\x10\xB2V[`\x03` R_\x90\x81R`@\x90 \x80T`\x01\x82\x01T`\x02\x90\x92\x01Ts\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x90\x91\x16\x91\x90`\xFF\x16\x83V[`@\x80Qs\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x90\x94\x16\x84R` \x84\x01\x92\x90\x92R\x15\x15\x90\x82\x01R``\x01a\x01\x1EV[a\x01ka\x02\x1D6`\x04a\x116V[a\x04\xBBV[a\x01Ja\x0206`\x04a\x10\xB2V[`\x02` R_\x90\x81R`@\x90 T\x81V[a\x01\x12a\x02O6`\x04a\x11\x0CV[_\x91\x82R`\x01` \x90\x81R`@\x80\x84 s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x93\x90\x93\x16\x84R\x91\x90R\x90 T`\xFF\x16\x90V[a\x01ka\x02\x946`\x04a\x11^V[a\x05\xBFV[a\x01ka\x02\xA76`\x04a\x10\xB2V[a\x07\x93V[a\x01J_\x81V[a\x01ka\x02\xC16`\x04a\x11\xDDV[a\t_V[a\x01ka\x02\xD46`\x04a\x11\x0CV[a\x0B2V[a\x01ka\x02\xE76`\x04a\x116V[a\x0BWV[a\x01ka\x02\xFA6`\x04a\x12IV[a\x0CQV[_\x7F\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x82\x16\x7Fye\xDB\x0B\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x14\x80a\x03\x91WP\x7F\x01\xFF\xC9\xA7\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x7F\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x83\x16\x14[\x92\x91PPV[_\x82\x81R`\x01` \x81\x90R`@\x90\x91 \x01Ta\x03\xB2\x81a\x0EaV[a\x03\xBC\x83\x83a\x0EnV[PPPPV[s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x163\x14a\x04\x11W`@Q\x7Ff\x97\xB22\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\x04\x1B\x82\x82a\x0F6V[PPPV[\x7F\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81\x16_\x90\x81R`\x02` R`@\x81 Ta\x04\x87W`@Q\x7F\x92^Z:\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[P\x7F\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x16_\x90\x81R`\x02` R`@\x90 T\x90V[\x7F\x13\x14\x10\xEA\xB1#l\xEE-\xB1\x905\xB0\xE8%\xC9NZ\xB7\x05\xDF\xFE#2\x1D\xD58V\xDAS\x16\x17a\x04\xE5\x81a\x0EaV[\x7F\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x83\x16_\x90\x81R`\x02` R`@\x90 T\x15a\x05MW`@Q\x7F\"\xA1\xBD\xC4\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x7F\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x83\x16_\x81\x81R`\x02` \x90\x81R`@\x91\x82\x90 \x85\x90U\x81Q\x92\x83R\x82\x01\x84\x90R\x7Fd\xB3R\x89tWJA\xDD\xC80\xAD\xC9d@\xDC\xCB\x99\xE6N\xDEk\x861\x94\xB9Q\xBF\xBD\x17\xE1\xBC\x91\x01[`@Q\x80\x91\x03\x90\xA1PPPV[\x7F\xBF\xD0\xF3\xF7\xFD\xD4\xEF\x11\x02\xD2\xA9\xFC\x8E\xBD\x96\x8FI\x9B\xDA\xDDYaS\xE0\"Z\xC2r\xF2O\xEC\x8Aa\x05\xE9\x81a\x0EaV[\x7F\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x84\x16a\x06BW`@Q\x7F \xAC\xD2\x8B\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x7F\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x84\x16_\x90\x81R`\x03` R`@\x90 \x80Ts\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x16\x15a\x06\xE1W\x80T`@Q\x7F+\x87\xE7\x97\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81Rs\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x90\x91\x16`\x04\x82\x01R`$\x01[`@Q\x80\x91\x03\x90\xFD[\x80T\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x16s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x85\x16\x90\x81\x17\x82U`\x01\x82\x01\x84\x90U`@\x80Q\x7F\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x88\x16\x81R` \x81\x01\x92\x90\x92R\x81\x01\x84\x90R\x7FLDU\xA3\x05(\xC3\x19d:\xE6\xFC5\xF3\xA0\xFC\xAB\xDEl\x01_*\x1ARp8,J\x19\x0B\x0F\xA3\x90``\x01[`@Q\x80\x91\x03\x90\xA1PPPPPV[\x7F\xC6\xF8\xFE\rW\x7F\xC3N\r\xCC\r\x9E\xF7\x9A\xEA\xA2\xD1f\xAF\x89\n\xA5'>\x16\x97\x04:\x05\\c\x16a\x07\xBD\x81a\x0EaV[\x7F\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x82\x16_\x90\x81R`\x03` R`@\x90 \x80Ts\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x16a\x08_W`@Q\x7F\xF2\x08w~\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R\x7F\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x84\x16`\x04\x82\x01R`$\x01a\x06\xD8V[`\x02\x81\x01T`\xFF\x16\x15a\x08\xC2W`@Q\x7F\xEB\xF1\x08#\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R\x7F\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x84\x16`\x04\x82\x01R`$\x01a\x06\xD8V[`\x02\x81\x01\x80T\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\0\x16`\x01\x17\x90U\x80T`@\x80Q\x7F\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x86\x16\x81Rs\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x90\x92\x16` \x83\x01R\x7Fc\xAD#c\xB1\x83\xCB\x8B\xB5b\xB9Y\x0C[D(\xE2\xA5f&\r\xF0S\xDB\x15ev\xD3\xD1qC\x8D\x91\x01a\x05\xB2V[_a\tm`\x04\x82\x84\x86a\x12\x9AV[a\tv\x91a\x12\xC1V[\x7F\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81\x16_\x90\x81R`\x03` \x90\x81R`@\x91\x82\x90 \x82Q``\x81\x01\x84R\x81Ts\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x16\x80\x82R`\x01\x83\x01T\x93\x82\x01\x93\x90\x93R`\x02\x90\x91\x01T`\xFF\x16\x15\x15\x92\x81\x01\x92\x90\x92R\x91\x92P\x90a\nIW`@Q\x7F\xF2\x08w~\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R\x7F\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x83\x16`\x04\x82\x01R`$\x01a\x06\xD8V[\x80`@\x01Q\x15a\n\xA9W`@Q\x7F\xEB\xF1\x08#\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R\x7F\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x83\x16`\x04\x82\x01R`$\x01a\x06\xD8V[\x80Q` \x82\x01Qs\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x90\x91\x16\x90cAI<`\x90\x88\x88a\n\xDE\x88`\x04\x81\x8Ca\x12\x9AV[`@Q\x86c\xFF\xFF\xFF\xFF\x16`\xE0\x1B\x81R`\x04\x01a\n\xFE\x95\x94\x93\x92\x91\x90a\x13nV[_`@Q\x80\x83\x03\x81\x86\x80;\x15\x80\x15a\x0B\x14W__\xFD[PZ\xFA\x15\x80\x15a\x0B&W=__>=_\xFD[PPPPPPPPPPV[_\x82\x81R`\x01` \x81\x90R`@\x90\x91 \x01Ta\x0BM\x81a\x0EaV[a\x03\xBC\x83\x83a\x0F6V[\x7F\x13\x14\x10\xEA\xB1#l\xEE-\xB1\x905\xB0\xE8%\xC9NZ\xB7\x05\xDF\xFE#2\x1D\xD58V\xDAS\x16\x17a\x0B\x81\x81a\x0EaV[\x7F\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x83\x16_\x90\x81R`\x02` R`@\x90 Ta\x0B\xE8W`@Q\x7F\x92^Z:\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x7F\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x83\x16_\x81\x81R`\x02` \x90\x81R`@\x91\x82\x90 \x85\x90U\x81Q\x92\x83R\x82\x01\x84\x90R\x7F=\x81Q\x8C\x99C\xE2\x9A\xF3\xAA|\x03u\x94\x82;-\xEE\x8D\x8A\x816\xD0\xEB%r\x1C\xEDH\xEBt\xE6\x91\x01a\x05\xB2V[_Ta\x01\0\x90\x04`\xFF\x16\x15\x80\x80\x15a\x0CoWP_T`\x01`\xFF\x90\x91\x16\x10[\x80a\x0C\x88WP0;\x15\x80\x15a\x0C\x88WP_T`\xFF\x16`\x01\x14[a\r\x14W`@Q\x7F\x08\xC3y\xA0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R` `\x04\x82\x01R`.`$\x82\x01R\x7FInitializable: contract is alrea`D\x82\x01R\x7Fdy initialized\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`d\x82\x01R`\x84\x01a\x06\xD8V[_\x80T\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\0\x16`\x01\x17\x90U\x80\x15a\rpW_\x80T\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\0\xFF\x16a\x01\0\x17\x90U[a\rz_\x86a\x0EnV[Pa\r\xA5\x7F\x13\x14\x10\xEA\xB1#l\xEE-\xB1\x905\xB0\xE8%\xC9NZ\xB7\x05\xDF\xFE#2\x1D\xD58V\xDAS\x16\x17\x85a\x0EnV[Pa\r\xD0\x7F\xBF\xD0\xF3\xF7\xFD\xD4\xEF\x11\x02\xD2\xA9\xFC\x8E\xBD\x96\x8FI\x9B\xDA\xDDYaS\xE0\"Z\xC2r\xF2O\xEC\x8A\x84a\x0EnV[Pa\r\xFB\x7F\xC6\xF8\xFE\rW\x7F\xC3N\r\xCC\r\x9E\xF7\x9A\xEA\xA2\xD1f\xAF\x89\n\xA5'>\x16\x97\x04:\x05\\c\x16\x83a\x0EnV[P\x80\x15a\x0EZW_\x80T\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\0\xFF\x16\x90U`@Q`\x01\x81R\x7F\x7F&\xB8?\xF9n\x1F+jh/\x138R\xF6y\x8A\t\xC4e\xDA\x95\x92\x14`\xCE\xFB8G@$\x98\x90` \x01a\x07\x84V[PPPPPV[a\x0Ek\x813a\x0F\xF3V[PV[_\x82\x81R`\x01` \x90\x81R`@\x80\x83 s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x85\x16\x84R\x90\x91R\x81 T`\xFF\x16a\x0F/W_\x83\x81R`\x01` \x81\x81R`@\x80\x84 s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x87\x16\x80\x86R\x92R\x80\x84 \x80T\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\0\x16\x90\x93\x17\x90\x92U\x90Q3\x92\x86\x91\x7F/\x87\x88\x11~~\xFF\x1D\x82\xE9&\xECyI\x01\xD1|x\x02JP'\t@0E@\xA73eo\r\x91\x90\xA4P`\x01a\x03\x91V[P_a\x03\x91V[_\x82\x81R`\x01` \x90\x81R`@\x80\x83 s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x85\x16\x84R\x90\x91R\x81 T`\xFF\x16\x15a\x0F/W_\x83\x81R`\x01` \x90\x81R`@\x80\x83 s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x86\x16\x80\x85R\x92R\x80\x83 \x80T\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\0\x16\x90UQ3\x92\x86\x91\x7F\xF69\x1F\\2\xD9\xC6\x9D*G\xEAg\x0BD)t\xB595\xD1\xED\xC7\xFDd\xEB!\xE0G\xA89\x17\x1B\x91\x90\xA4P`\x01a\x03\x91V[_\x82\x81R`\x01` \x90\x81R`@\x80\x83 s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x85\x16\x84R\x90\x91R\x90 T`\xFF\x16a\x10zW`@Q\x7F\xE2Q}?\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81Rs\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x16`\x04\x82\x01R`$\x81\x01\x83\x90R`D\x01a\x06\xD8V[PPV[\x805\x7F\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81\x16\x81\x14a\x10\xADW__\xFD[\x91\x90PV[_` \x82\x84\x03\x12\x15a\x10\xC2W__\xFD[a\x10\xCB\x82a\x10~V[\x93\x92PPPV[_` \x82\x84\x03\x12\x15a\x10\xE2W__\xFD[P5\x91\x90PV[\x805s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x16\x81\x14a\x10\xADW__\xFD[__`@\x83\x85\x03\x12\x15a\x11\x1DW__\xFD[\x825\x91Pa\x11-` \x84\x01a\x10\xE9V[\x90P\x92P\x92\x90PV[__`@\x83\x85\x03\x12\x15a\x11GW__\xFD[a\x11P\x83a\x10~V[\x94` \x93\x90\x93\x015\x93PPPV[___``\x84\x86\x03\x12\x15a\x11pW__\xFD[a\x11y\x84a\x10~V[\x92Pa\x11\x87` \x85\x01a\x10\xE9V[\x92\x95\x92\x94PPP`@\x91\x90\x91\x015\x90V[__\x83`\x1F\x84\x01\x12a\x11\xA8W__\xFD[P\x815g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x11\xBFW__\xFD[` \x83\x01\x91P\x83` \x82\x85\x01\x01\x11\x15a\x11\xD6W__\xFD[\x92P\x92\x90PV[____`@\x85\x87\x03\x12\x15a\x11\xF0W__\xFD[\x845g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x12\x06W__\xFD[a\x12\x12\x87\x82\x88\x01a\x11\x98V[\x90\x95P\x93PP` \x85\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x121W__\xFD[a\x12=\x87\x82\x88\x01a\x11\x98V[\x95\x98\x94\x97P\x95PPPPV[____`\x80\x85\x87\x03\x12\x15a\x12\\W__\xFD[a\x12e\x85a\x10\xE9V[\x93Pa\x12s` \x86\x01a\x10\xE9V[\x92Pa\x12\x81`@\x86\x01a\x10\xE9V[\x91Pa\x12\x8F``\x86\x01a\x10\xE9V[\x90P\x92\x95\x91\x94P\x92PV[__\x85\x85\x11\x15a\x12\xA8W__\xFD[\x83\x86\x11\x15a\x12\xB4W__\xFD[PP\x82\x01\x93\x91\x90\x92\x03\x91PV[\x805\x7F\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81\x16\x90`\x04\x84\x10\x15a\x13 W\x7F\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x80\x85`\x04\x03`\x03\x1B\x1B\x82\x16\x16\x91P[P\x92\x91PPV[\x81\x83R\x81\x81` \x85\x017P_` \x82\x84\x01\x01R_` \x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xE0`\x1F\x84\x01\x16\x84\x01\x01\x90P\x92\x91PPV[\x85\x81R``` \x82\x01R_a\x13\x87``\x83\x01\x86\x88a\x13'V[\x82\x81\x03`@\x84\x01Ra\x13\x9A\x81\x85\x87a\x13'V[\x98\x97PPPPPPPPV\xFE\xA2dipfsX\"\x12 \x9C\xA5`\xA4<\xC6\xB7A\xEAd\xD3\xD9\x9AxEQ\rL\xC7\xBE\xA2+3\xBF\x0C\xED\x7F\x15\xFFr6ZdsolcC\0\x08\x1C\x003";
    /// The bytecode of the contract.
    pub static AGGLAYERGATEWAY_BYTECODE: ::ethers::core::types::Bytes = ::ethers::core::types::Bytes::from_static(
        __BYTECODE,
    );
    #[rustfmt::skip]
    const __DEPLOYED_BYTECODE: &[u8] = b"`\x80`@R4\x80\x15a\0\x0FW__\xFD[P`\x046\x10a\0\xFBW_5`\xE0\x1C\x80c\x91\xD1HT\x11a\0\x93W\x80c\xA4\x8F\xD3w\x11a\0cW\x80c\xA4\x8F\xD3w\x14a\x02\xB3W\x80c\xD5Gt\x1F\x14a\x02\xC6W\x80c\xF4\xC4F\x81\x14a\x02\xD9W\x80c\xF8\xC8v^\x14a\x02\xECW__\xFD[\x80c\x91\xD1HT\x14a\x02AW\x80c\x95\x84\xA5\x16\x14a\x02\x86W\x80c\x95\xD2\xA3\x91\x14a\x02\x99W\x80c\xA2\x17\xFD\xDF\x14a\x02\xACW__\xFD[\x80cl\xAB\xFD\xAB\x11a\0\xCEW\x80cl\xAB\xFD\xAB\x14a\x01\x80W\x80cqk\xE0u\x14a\x01\x93W\x80c\x81\x8C\x8C!\x14a\x02\x0FW\x80c\x82\xBF\xAE\xA1\x14a\x02\"W__\xFD[\x80c\x01\xFF\xC9\xA7\x14a\0\xFFW\x80c$\x8A\x9C\xA3\x14a\x01'W\x80c//\xF1]\x14a\x01XW\x80c6V\x8A\xBE\x14a\x01mW[__\xFD[a\x01\x12a\x01\r6`\x04a\x10\xB2V[a\x02\xFFV[`@Q\x90\x15\x15\x81R` \x01[`@Q\x80\x91\x03\x90\xF3[a\x01Ja\x0156`\x04a\x10\xD2V[_\x90\x81R`\x01` \x81\x90R`@\x90\x91 \x01T\x90V[`@Q\x90\x81R` \x01a\x01\x1EV[a\x01ka\x01f6`\x04a\x11\x0CV[a\x03\x97V[\0[a\x01ka\x01{6`\x04a\x11\x0CV[a\x03\xC2V[a\x01Ja\x01\x8E6`\x04a\x10\xB2V[a\x04 V[a\x01\xDBa\x01\xA16`\x04a\x10\xB2V[`\x03` R_\x90\x81R`@\x90 \x80T`\x01\x82\x01T`\x02\x90\x92\x01Ts\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x90\x91\x16\x91\x90`\xFF\x16\x83V[`@\x80Qs\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x90\x94\x16\x84R` \x84\x01\x92\x90\x92R\x15\x15\x90\x82\x01R``\x01a\x01\x1EV[a\x01ka\x02\x1D6`\x04a\x116V[a\x04\xBBV[a\x01Ja\x0206`\x04a\x10\xB2V[`\x02` R_\x90\x81R`@\x90 T\x81V[a\x01\x12a\x02O6`\x04a\x11\x0CV[_\x91\x82R`\x01` \x90\x81R`@\x80\x84 s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x93\x90\x93\x16\x84R\x91\x90R\x90 T`\xFF\x16\x90V[a\x01ka\x02\x946`\x04a\x11^V[a\x05\xBFV[a\x01ka\x02\xA76`\x04a\x10\xB2V[a\x07\x93V[a\x01J_\x81V[a\x01ka\x02\xC16`\x04a\x11\xDDV[a\t_V[a\x01ka\x02\xD46`\x04a\x11\x0CV[a\x0B2V[a\x01ka\x02\xE76`\x04a\x116V[a\x0BWV[a\x01ka\x02\xFA6`\x04a\x12IV[a\x0CQV[_\x7F\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x82\x16\x7Fye\xDB\x0B\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x14\x80a\x03\x91WP\x7F\x01\xFF\xC9\xA7\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x7F\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x83\x16\x14[\x92\x91PPV[_\x82\x81R`\x01` \x81\x90R`@\x90\x91 \x01Ta\x03\xB2\x81a\x0EaV[a\x03\xBC\x83\x83a\x0EnV[PPPPV[s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x163\x14a\x04\x11W`@Q\x7Ff\x97\xB22\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[a\x04\x1B\x82\x82a\x0F6V[PPPV[\x7F\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81\x16_\x90\x81R`\x02` R`@\x81 Ta\x04\x87W`@Q\x7F\x92^Z:\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[P\x7F\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x16_\x90\x81R`\x02` R`@\x90 T\x90V[\x7F\x13\x14\x10\xEA\xB1#l\xEE-\xB1\x905\xB0\xE8%\xC9NZ\xB7\x05\xDF\xFE#2\x1D\xD58V\xDAS\x16\x17a\x04\xE5\x81a\x0EaV[\x7F\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x83\x16_\x90\x81R`\x02` R`@\x90 T\x15a\x05MW`@Q\x7F\"\xA1\xBD\xC4\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x7F\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x83\x16_\x81\x81R`\x02` \x90\x81R`@\x91\x82\x90 \x85\x90U\x81Q\x92\x83R\x82\x01\x84\x90R\x7Fd\xB3R\x89tWJA\xDD\xC80\xAD\xC9d@\xDC\xCB\x99\xE6N\xDEk\x861\x94\xB9Q\xBF\xBD\x17\xE1\xBC\x91\x01[`@Q\x80\x91\x03\x90\xA1PPPV[\x7F\xBF\xD0\xF3\xF7\xFD\xD4\xEF\x11\x02\xD2\xA9\xFC\x8E\xBD\x96\x8FI\x9B\xDA\xDDYaS\xE0\"Z\xC2r\xF2O\xEC\x8Aa\x05\xE9\x81a\x0EaV[\x7F\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x84\x16a\x06BW`@Q\x7F \xAC\xD2\x8B\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x7F\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x84\x16_\x90\x81R`\x03` R`@\x90 \x80Ts\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x16\x15a\x06\xE1W\x80T`@Q\x7F+\x87\xE7\x97\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81Rs\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x90\x91\x16`\x04\x82\x01R`$\x01[`@Q\x80\x91\x03\x90\xFD[\x80T\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x16s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x85\x16\x90\x81\x17\x82U`\x01\x82\x01\x84\x90U`@\x80Q\x7F\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x88\x16\x81R` \x81\x01\x92\x90\x92R\x81\x01\x84\x90R\x7FLDU\xA3\x05(\xC3\x19d:\xE6\xFC5\xF3\xA0\xFC\xAB\xDEl\x01_*\x1ARp8,J\x19\x0B\x0F\xA3\x90``\x01[`@Q\x80\x91\x03\x90\xA1PPPPPV[\x7F\xC6\xF8\xFE\rW\x7F\xC3N\r\xCC\r\x9E\xF7\x9A\xEA\xA2\xD1f\xAF\x89\n\xA5'>\x16\x97\x04:\x05\\c\x16a\x07\xBD\x81a\x0EaV[\x7F\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x82\x16_\x90\x81R`\x03` R`@\x90 \x80Ts\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x16a\x08_W`@Q\x7F\xF2\x08w~\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R\x7F\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x84\x16`\x04\x82\x01R`$\x01a\x06\xD8V[`\x02\x81\x01T`\xFF\x16\x15a\x08\xC2W`@Q\x7F\xEB\xF1\x08#\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R\x7F\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x84\x16`\x04\x82\x01R`$\x01a\x06\xD8V[`\x02\x81\x01\x80T\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\0\x16`\x01\x17\x90U\x80T`@\x80Q\x7F\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x86\x16\x81Rs\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x90\x92\x16` \x83\x01R\x7Fc\xAD#c\xB1\x83\xCB\x8B\xB5b\xB9Y\x0C[D(\xE2\xA5f&\r\xF0S\xDB\x15ev\xD3\xD1qC\x8D\x91\x01a\x05\xB2V[_a\tm`\x04\x82\x84\x86a\x12\x9AV[a\tv\x91a\x12\xC1V[\x7F\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81\x16_\x90\x81R`\x03` \x90\x81R`@\x91\x82\x90 \x82Q``\x81\x01\x84R\x81Ts\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x16\x80\x82R`\x01\x83\x01T\x93\x82\x01\x93\x90\x93R`\x02\x90\x91\x01T`\xFF\x16\x15\x15\x92\x81\x01\x92\x90\x92R\x91\x92P\x90a\nIW`@Q\x7F\xF2\x08w~\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R\x7F\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x83\x16`\x04\x82\x01R`$\x01a\x06\xD8V[\x80`@\x01Q\x15a\n\xA9W`@Q\x7F\xEB\xF1\x08#\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R\x7F\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x83\x16`\x04\x82\x01R`$\x01a\x06\xD8V[\x80Q` \x82\x01Qs\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x90\x91\x16\x90cAI<`\x90\x88\x88a\n\xDE\x88`\x04\x81\x8Ca\x12\x9AV[`@Q\x86c\xFF\xFF\xFF\xFF\x16`\xE0\x1B\x81R`\x04\x01a\n\xFE\x95\x94\x93\x92\x91\x90a\x13nV[_`@Q\x80\x83\x03\x81\x86\x80;\x15\x80\x15a\x0B\x14W__\xFD[PZ\xFA\x15\x80\x15a\x0B&W=__>=_\xFD[PPPPPPPPPPV[_\x82\x81R`\x01` \x81\x90R`@\x90\x91 \x01Ta\x0BM\x81a\x0EaV[a\x03\xBC\x83\x83a\x0F6V[\x7F\x13\x14\x10\xEA\xB1#l\xEE-\xB1\x905\xB0\xE8%\xC9NZ\xB7\x05\xDF\xFE#2\x1D\xD58V\xDAS\x16\x17a\x0B\x81\x81a\x0EaV[\x7F\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x83\x16_\x90\x81R`\x02` R`@\x90 Ta\x0B\xE8W`@Q\x7F\x92^Z:\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[\x7F\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x83\x16_\x81\x81R`\x02` \x90\x81R`@\x91\x82\x90 \x85\x90U\x81Q\x92\x83R\x82\x01\x84\x90R\x7F=\x81Q\x8C\x99C\xE2\x9A\xF3\xAA|\x03u\x94\x82;-\xEE\x8D\x8A\x816\xD0\xEB%r\x1C\xEDH\xEBt\xE6\x91\x01a\x05\xB2V[_Ta\x01\0\x90\x04`\xFF\x16\x15\x80\x80\x15a\x0CoWP_T`\x01`\xFF\x90\x91\x16\x10[\x80a\x0C\x88WP0;\x15\x80\x15a\x0C\x88WP_T`\xFF\x16`\x01\x14[a\r\x14W`@Q\x7F\x08\xC3y\xA0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R` `\x04\x82\x01R`.`$\x82\x01R\x7FInitializable: contract is alrea`D\x82\x01R\x7Fdy initialized\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`d\x82\x01R`\x84\x01a\x06\xD8V[_\x80T\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\0\x16`\x01\x17\x90U\x80\x15a\rpW_\x80T\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\0\xFF\x16a\x01\0\x17\x90U[a\rz_\x86a\x0EnV[Pa\r\xA5\x7F\x13\x14\x10\xEA\xB1#l\xEE-\xB1\x905\xB0\xE8%\xC9NZ\xB7\x05\xDF\xFE#2\x1D\xD58V\xDAS\x16\x17\x85a\x0EnV[Pa\r\xD0\x7F\xBF\xD0\xF3\xF7\xFD\xD4\xEF\x11\x02\xD2\xA9\xFC\x8E\xBD\x96\x8FI\x9B\xDA\xDDYaS\xE0\"Z\xC2r\xF2O\xEC\x8A\x84a\x0EnV[Pa\r\xFB\x7F\xC6\xF8\xFE\rW\x7F\xC3N\r\xCC\r\x9E\xF7\x9A\xEA\xA2\xD1f\xAF\x89\n\xA5'>\x16\x97\x04:\x05\\c\x16\x83a\x0EnV[P\x80\x15a\x0EZW_\x80T\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\0\xFF\x16\x90U`@Q`\x01\x81R\x7F\x7F&\xB8?\xF9n\x1F+jh/\x138R\xF6y\x8A\t\xC4e\xDA\x95\x92\x14`\xCE\xFB8G@$\x98\x90` \x01a\x07\x84V[PPPPPV[a\x0Ek\x813a\x0F\xF3V[PV[_\x82\x81R`\x01` \x90\x81R`@\x80\x83 s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x85\x16\x84R\x90\x91R\x81 T`\xFF\x16a\x0F/W_\x83\x81R`\x01` \x81\x81R`@\x80\x84 s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x87\x16\x80\x86R\x92R\x80\x84 \x80T\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\0\x16\x90\x93\x17\x90\x92U\x90Q3\x92\x86\x91\x7F/\x87\x88\x11~~\xFF\x1D\x82\xE9&\xECyI\x01\xD1|x\x02JP'\t@0E@\xA73eo\r\x91\x90\xA4P`\x01a\x03\x91V[P_a\x03\x91V[_\x82\x81R`\x01` \x90\x81R`@\x80\x83 s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x85\x16\x84R\x90\x91R\x81 T`\xFF\x16\x15a\x0F/W_\x83\x81R`\x01` \x90\x81R`@\x80\x83 s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x86\x16\x80\x85R\x92R\x80\x83 \x80T\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\0\x16\x90UQ3\x92\x86\x91\x7F\xF69\x1F\\2\xD9\xC6\x9D*G\xEAg\x0BD)t\xB595\xD1\xED\xC7\xFDd\xEB!\xE0G\xA89\x17\x1B\x91\x90\xA4P`\x01a\x03\x91V[_\x82\x81R`\x01` \x90\x81R`@\x80\x83 s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x85\x16\x84R\x90\x91R\x90 T`\xFF\x16a\x10zW`@Q\x7F\xE2Q}?\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81Rs\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x16`\x04\x82\x01R`$\x81\x01\x83\x90R`D\x01a\x06\xD8V[PPV[\x805\x7F\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81\x16\x81\x14a\x10\xADW__\xFD[\x91\x90PV[_` \x82\x84\x03\x12\x15a\x10\xC2W__\xFD[a\x10\xCB\x82a\x10~V[\x93\x92PPPV[_` \x82\x84\x03\x12\x15a\x10\xE2W__\xFD[P5\x91\x90PV[\x805s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x16\x81\x14a\x10\xADW__\xFD[__`@\x83\x85\x03\x12\x15a\x11\x1DW__\xFD[\x825\x91Pa\x11-` \x84\x01a\x10\xE9V[\x90P\x92P\x92\x90PV[__`@\x83\x85\x03\x12\x15a\x11GW__\xFD[a\x11P\x83a\x10~V[\x94` \x93\x90\x93\x015\x93PPPV[___``\x84\x86\x03\x12\x15a\x11pW__\xFD[a\x11y\x84a\x10~V[\x92Pa\x11\x87` \x85\x01a\x10\xE9V[\x92\x95\x92\x94PPP`@\x91\x90\x91\x015\x90V[__\x83`\x1F\x84\x01\x12a\x11\xA8W__\xFD[P\x815g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x11\xBFW__\xFD[` \x83\x01\x91P\x83` \x82\x85\x01\x01\x11\x15a\x11\xD6W__\xFD[\x92P\x92\x90PV[____`@\x85\x87\x03\x12\x15a\x11\xF0W__\xFD[\x845g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x12\x06W__\xFD[a\x12\x12\x87\x82\x88\x01a\x11\x98V[\x90\x95P\x93PP` \x85\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x121W__\xFD[a\x12=\x87\x82\x88\x01a\x11\x98V[\x95\x98\x94\x97P\x95PPPPV[____`\x80\x85\x87\x03\x12\x15a\x12\\W__\xFD[a\x12e\x85a\x10\xE9V[\x93Pa\x12s` \x86\x01a\x10\xE9V[\x92Pa\x12\x81`@\x86\x01a\x10\xE9V[\x91Pa\x12\x8F``\x86\x01a\x10\xE9V[\x90P\x92\x95\x91\x94P\x92PV[__\x85\x85\x11\x15a\x12\xA8W__\xFD[\x83\x86\x11\x15a\x12\xB4W__\xFD[PP\x82\x01\x93\x91\x90\x92\x03\x91PV[\x805\x7F\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81\x16\x90`\x04\x84\x10\x15a\x13 W\x7F\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x80\x85`\x04\x03`\x03\x1B\x1B\x82\x16\x16\x91P[P\x92\x91PPV[\x81\x83R\x81\x81` \x85\x017P_` \x82\x84\x01\x01R_` \x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xE0`\x1F\x84\x01\x16\x84\x01\x01\x90P\x92\x91PPV[\x85\x81R``` \x82\x01R_a\x13\x87``\x83\x01\x86\x88a\x13'V[\x82\x81\x03`@\x84\x01Ra\x13\x9A\x81\x85\x87a\x13'V[\x98\x97PPPPPPPPV\xFE\xA2dipfsX\"\x12 \x9C\xA5`\xA4<\xC6\xB7A\xEAd\xD3\xD9\x9AxEQ\rL\xC7\xBE\xA2+3\xBF\x0C\xED\x7F\x15\xFFr6ZdsolcC\0\x08\x1C\x003";
    /// The deployed bytecode of the contract.
    pub static AGGLAYERGATEWAY_DEPLOYED_BYTECODE: ::ethers::core::types::Bytes = ::ethers::core::types::Bytes::from_static(
        __DEPLOYED_BYTECODE,
    );
    pub struct AggLayerGateway<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for AggLayerGateway<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for AggLayerGateway<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for AggLayerGateway<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for AggLayerGateway<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(::core::stringify!(AggLayerGateway))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> AggLayerGateway<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(
                ::ethers::contract::Contract::new(
                    address.into(),
                    AGGLAYERGATEWAY_ABI.clone(),
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
                AGGLAYERGATEWAY_ABI.clone(),
                AGGLAYERGATEWAY_BYTECODE.clone().into(),
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
        ///Calls the contract's `addDefaultAggchainVKey` (0x818c8c21) function
        pub fn add_default_aggchain_v_key(
            &self,
            default_aggchain_selector: [u8; 4],
            new_aggchain_v_key: [u8; 32],
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [129, 140, 140, 33],
                    (default_aggchain_selector, new_aggchain_v_key),
                )
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `addPessimisticVKeyRoute` (0x9584a516) function
        pub fn add_pessimistic_v_key_route(
            &self,
            pessimistic_v_key_selector: [u8; 4],
            verifier: ::ethers::core::types::Address,
            pessimistic_v_key: [u8; 32],
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [149, 132, 165, 22],
                    (pessimistic_v_key_selector, verifier, pessimistic_v_key),
                )
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `defaultAggchainVKeys` (0x82bfaea1) function
        pub fn default_aggchain_v_keys(
            &self,
            default_aggchain_selector: [u8; 4],
        ) -> ::ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([130, 191, 174, 161], default_aggchain_selector)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `freezePessimisticVKeyRoute` (0x95d2a391) function
        pub fn freeze_pessimistic_v_key_route(
            &self,
            pessimistic_v_key_selector: [u8; 4],
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([149, 210, 163, 145], pessimistic_v_key_selector)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getDefaultAggchainVKey` (0x6cabfdab) function
        pub fn get_default_aggchain_v_key(
            &self,
            default_aggchain_selector: [u8; 4],
        ) -> ::ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([108, 171, 253, 171], default_aggchain_selector)
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
        ///Calls the contract's `initialize` (0xf8c8765e) function
        pub fn initialize(
            &self,
            default_admin: ::ethers::core::types::Address,
            aggchain_default_v_key_role: ::ethers::core::types::Address,
            add_route_role: ::ethers::core::types::Address,
            freeze_route_role: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [248, 200, 118, 94],
                    (
                        default_admin,
                        aggchain_default_v_key_role,
                        add_route_role,
                        freeze_route_role,
                    ),
                )
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `pessimisticVKeyRoutes` (0x716be075) function
        pub fn pessimistic_v_key_routes(
            &self,
            pessimistic_v_key_selector: [u8; 4],
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            (::ethers::core::types::Address, [u8; 32], bool),
        > {
            self.0
                .method_hash([113, 107, 224, 117], pessimistic_v_key_selector)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `renounceRole` (0x36568abe) function
        pub fn renounce_role(
            &self,
            role: [u8; 32],
            caller_confirmation: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([54, 86, 138, 190], (role, caller_confirmation))
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
        ///Calls the contract's `supportsInterface` (0x01ffc9a7) function
        pub fn supports_interface(
            &self,
            interface_id: [u8; 4],
        ) -> ::ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([1, 255, 201, 167], interface_id)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `updateDefaultAggchainVKey` (0xf4c44681) function
        pub fn update_default_aggchain_v_key(
            &self,
            default_aggchain_selector: [u8; 4],
            new_default_aggchain_v_key: [u8; 32],
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [244, 196, 70, 129],
                    (default_aggchain_selector, new_default_aggchain_v_key),
                )
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `verifyPessimisticProof` (0xa48fd377) function
        pub fn verify_pessimistic_proof(
            &self,
            public_values: ::ethers::core::types::Bytes,
            proof_bytes: ::ethers::core::types::Bytes,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([164, 143, 211, 119], (public_values, proof_bytes))
                .expect("method not found (this should never happen)")
        }
        ///Gets the contract's `AcceptAggLayerAdminRole` event
        pub fn accept_agg_layer_admin_role_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            AcceptAggLayerAdminRoleFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `AddDefaultAggchainVKey` event
        pub fn add_default_aggchain_v_key_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            AddDefaultAggchainVKeyFilter,
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
        ///Gets the contract's `RouteAdded` event
        pub fn route_added_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            RouteAddedFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `RouteFrozen` event
        pub fn route_frozen_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            RouteFrozenFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `TransferAggLayerAdminRole` event
        pub fn transfer_agg_layer_admin_role_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            TransferAggLayerAdminRoleFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `UpdateDefaultAggchainVKey` event
        pub fn update_default_aggchain_v_key_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            UpdateDefaultAggchainVKeyFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `UpdatePessimisticVKey` event
        pub fn update_pessimistic_v_key_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            UpdatePessimisticVKeyFilter,
        > {
            self.0.event()
        }
        /// Returns an `Event` builder for all the events of this contract.
        pub fn events(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            AggLayerGatewayEvents,
        > {
            self.0.event_with_filter(::core::default::Default::default())
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
    for AggLayerGateway<M> {
        fn from(contract: ::ethers::contract::Contract<M>) -> Self {
            Self::new(contract.address(), contract.client())
        }
    }
    ///Custom Error type `AccessControlBadConfirmation` with signature `AccessControlBadConfirmation()` and selector `0x6697b232`
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
        name = "AccessControlBadConfirmation",
        abi = "AccessControlBadConfirmation()"
    )]
    pub struct AccessControlBadConfirmation;
    ///Custom Error type `AccessControlUnauthorizedAccount` with signature `AccessControlUnauthorizedAccount(address,bytes32)` and selector `0xe2517d3f`
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
        name = "AccessControlUnauthorizedAccount",
        abi = "AccessControlUnauthorizedAccount(address,bytes32)"
    )]
    pub struct AccessControlUnauthorizedAccount {
        pub account: ::ethers::core::types::Address,
        pub needed_role: [u8; 32],
    }
    ///Custom Error type `AggchainVKeyAlreadyExists` with signature `AggchainVKeyAlreadyExists()` and selector `0x22a1bdc4`
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
    #[etherror(name = "AggchainVKeyAlreadyExists", abi = "AggchainVKeyAlreadyExists()")]
    pub struct AggchainVKeyAlreadyExists;
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
    ///Custom Error type `OnlyAggLayerAdmin` with signature `OnlyAggLayerAdmin()` and selector `0x4c939ed7`
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
    #[etherror(name = "OnlyAggLayerAdmin", abi = "OnlyAggLayerAdmin()")]
    pub struct OnlyAggLayerAdmin;
    ///Custom Error type `OnlyPendingAggLayerAdmin` with signature `OnlyPendingAggLayerAdmin()` and selector `0xce074d87`
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
    #[etherror(name = "OnlyPendingAggLayerAdmin", abi = "OnlyPendingAggLayerAdmin()")]
    pub struct OnlyPendingAggLayerAdmin;
    ///Custom Error type `RouteAlreadyExists` with signature `RouteAlreadyExists(address)` and selector `0x2b87e797`
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
    #[etherror(name = "RouteAlreadyExists", abi = "RouteAlreadyExists(address)")]
    pub struct RouteAlreadyExists {
        pub verifier: ::ethers::core::types::Address,
    }
    ///Custom Error type `RouteIsFrozen` with signature `RouteIsFrozen(bytes4)` and selector `0xebf10823`
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
    #[etherror(name = "RouteIsFrozen", abi = "RouteIsFrozen(bytes4)")]
    pub struct RouteIsFrozen {
        pub selector: [u8; 4],
    }
    ///Custom Error type `RouteNotFound` with signature `RouteNotFound(bytes4)` and selector `0xf208777e`
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
    #[etherror(name = "RouteNotFound", abi = "RouteNotFound(bytes4)")]
    pub struct RouteNotFound {
        pub selector: [u8; 4],
    }
    ///Custom Error type `SelectorCannotBeZero` with signature `SelectorCannotBeZero()` and selector `0x20acd28b`
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
    #[etherror(name = "SelectorCannotBeZero", abi = "SelectorCannotBeZero()")]
    pub struct SelectorCannotBeZero;
    ///Container type for all of the contract's custom errors
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum AggLayerGatewayErrors {
        AccessControlBadConfirmation(AccessControlBadConfirmation),
        AccessControlUnauthorizedAccount(AccessControlUnauthorizedAccount),
        AggchainVKeyAlreadyExists(AggchainVKeyAlreadyExists),
        AggchainVKeyNotFound(AggchainVKeyNotFound),
        OnlyAggLayerAdmin(OnlyAggLayerAdmin),
        OnlyPendingAggLayerAdmin(OnlyPendingAggLayerAdmin),
        RouteAlreadyExists(RouteAlreadyExists),
        RouteIsFrozen(RouteIsFrozen),
        RouteNotFound(RouteNotFound),
        SelectorCannotBeZero(SelectorCannotBeZero),
        /// The standard solidity revert string, with selector
        /// Error(string) -- 0x08c379a0
        RevertString(::std::string::String),
    }
    impl ::ethers::core::abi::AbiDecode for AggLayerGatewayErrors {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded) = <::std::string::String as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::RevertString(decoded));
            }
            if let Ok(decoded) = <AccessControlBadConfirmation as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::AccessControlBadConfirmation(decoded));
            }
            if let Ok(decoded) = <AccessControlUnauthorizedAccount as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::AccessControlUnauthorizedAccount(decoded));
            }
            if let Ok(decoded) = <AggchainVKeyAlreadyExists as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::AggchainVKeyAlreadyExists(decoded));
            }
            if let Ok(decoded) = <AggchainVKeyNotFound as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::AggchainVKeyNotFound(decoded));
            }
            if let Ok(decoded) = <OnlyAggLayerAdmin as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::OnlyAggLayerAdmin(decoded));
            }
            if let Ok(decoded) = <OnlyPendingAggLayerAdmin as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::OnlyPendingAggLayerAdmin(decoded));
            }
            if let Ok(decoded) = <RouteAlreadyExists as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::RouteAlreadyExists(decoded));
            }
            if let Ok(decoded) = <RouteIsFrozen as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::RouteIsFrozen(decoded));
            }
            if let Ok(decoded) = <RouteNotFound as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::RouteNotFound(decoded));
            }
            if let Ok(decoded) = <SelectorCannotBeZero as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::SelectorCannotBeZero(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for AggLayerGatewayErrors {
        fn encode(self) -> ::std::vec::Vec<u8> {
            match self {
                Self::AccessControlBadConfirmation(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::AccessControlUnauthorizedAccount(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::AggchainVKeyAlreadyExists(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::AggchainVKeyNotFound(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::OnlyAggLayerAdmin(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::OnlyPendingAggLayerAdmin(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RouteAlreadyExists(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RouteIsFrozen(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RouteNotFound(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::SelectorCannotBeZero(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RevertString(s) => ::ethers::core::abi::AbiEncode::encode(s),
            }
        }
    }
    impl ::ethers::contract::ContractRevert for AggLayerGatewayErrors {
        fn valid_selector(selector: [u8; 4]) -> bool {
            match selector {
                [0x08, 0xc3, 0x79, 0xa0] => true,
                _ if selector
                    == <AccessControlBadConfirmation as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <AccessControlUnauthorizedAccount as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <AggchainVKeyAlreadyExists as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <AggchainVKeyNotFound as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <OnlyAggLayerAdmin as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <OnlyPendingAggLayerAdmin as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <RouteAlreadyExists as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <RouteIsFrozen as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <RouteNotFound as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <SelectorCannotBeZero as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ => false,
            }
        }
    }
    impl ::core::fmt::Display for AggLayerGatewayErrors {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::AccessControlBadConfirmation(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::AccessControlUnauthorizedAccount(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::AggchainVKeyAlreadyExists(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::AggchainVKeyNotFound(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::OnlyAggLayerAdmin(element) => ::core::fmt::Display::fmt(element, f),
                Self::OnlyPendingAggLayerAdmin(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RouteAlreadyExists(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RouteIsFrozen(element) => ::core::fmt::Display::fmt(element, f),
                Self::RouteNotFound(element) => ::core::fmt::Display::fmt(element, f),
                Self::SelectorCannotBeZero(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RevertString(s) => ::core::fmt::Display::fmt(s, f),
            }
        }
    }
    impl ::core::convert::From<::std::string::String> for AggLayerGatewayErrors {
        fn from(value: String) -> Self {
            Self::RevertString(value)
        }
    }
    impl ::core::convert::From<AccessControlBadConfirmation> for AggLayerGatewayErrors {
        fn from(value: AccessControlBadConfirmation) -> Self {
            Self::AccessControlBadConfirmation(value)
        }
    }
    impl ::core::convert::From<AccessControlUnauthorizedAccount>
    for AggLayerGatewayErrors {
        fn from(value: AccessControlUnauthorizedAccount) -> Self {
            Self::AccessControlUnauthorizedAccount(value)
        }
    }
    impl ::core::convert::From<AggchainVKeyAlreadyExists> for AggLayerGatewayErrors {
        fn from(value: AggchainVKeyAlreadyExists) -> Self {
            Self::AggchainVKeyAlreadyExists(value)
        }
    }
    impl ::core::convert::From<AggchainVKeyNotFound> for AggLayerGatewayErrors {
        fn from(value: AggchainVKeyNotFound) -> Self {
            Self::AggchainVKeyNotFound(value)
        }
    }
    impl ::core::convert::From<OnlyAggLayerAdmin> for AggLayerGatewayErrors {
        fn from(value: OnlyAggLayerAdmin) -> Self {
            Self::OnlyAggLayerAdmin(value)
        }
    }
    impl ::core::convert::From<OnlyPendingAggLayerAdmin> for AggLayerGatewayErrors {
        fn from(value: OnlyPendingAggLayerAdmin) -> Self {
            Self::OnlyPendingAggLayerAdmin(value)
        }
    }
    impl ::core::convert::From<RouteAlreadyExists> for AggLayerGatewayErrors {
        fn from(value: RouteAlreadyExists) -> Self {
            Self::RouteAlreadyExists(value)
        }
    }
    impl ::core::convert::From<RouteIsFrozen> for AggLayerGatewayErrors {
        fn from(value: RouteIsFrozen) -> Self {
            Self::RouteIsFrozen(value)
        }
    }
    impl ::core::convert::From<RouteNotFound> for AggLayerGatewayErrors {
        fn from(value: RouteNotFound) -> Self {
            Self::RouteNotFound(value)
        }
    }
    impl ::core::convert::From<SelectorCannotBeZero> for AggLayerGatewayErrors {
        fn from(value: SelectorCannotBeZero) -> Self {
            Self::SelectorCannotBeZero(value)
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
        name = "AcceptAggLayerAdminRole",
        abi = "AcceptAggLayerAdminRole(address)"
    )]
    pub struct AcceptAggLayerAdminRoleFilter {
        pub new_agg_layer_admin: ::ethers::core::types::Address,
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
        name = "AddDefaultAggchainVKey",
        abi = "AddDefaultAggchainVKey(bytes4,bytes32)"
    )]
    pub struct AddDefaultAggchainVKeyFilter {
        pub selector: [u8; 4],
        pub new_v_key: [u8; 32],
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
    #[ethevent(name = "RouteAdded", abi = "RouteAdded(bytes4,address,bytes32)")]
    pub struct RouteAddedFilter {
        pub selector: [u8; 4],
        pub verifier: ::ethers::core::types::Address,
        pub pessimistic_v_key: [u8; 32],
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
    #[ethevent(name = "RouteFrozen", abi = "RouteFrozen(bytes4,address)")]
    pub struct RouteFrozenFilter {
        pub selector: [u8; 4],
        pub verifier: ::ethers::core::types::Address,
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
        name = "TransferAggLayerAdminRole",
        abi = "TransferAggLayerAdminRole(address)"
    )]
    pub struct TransferAggLayerAdminRoleFilter {
        pub new_pending_agg_layer_admin: ::ethers::core::types::Address,
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
        name = "UpdateDefaultAggchainVKey",
        abi = "UpdateDefaultAggchainVKey(bytes4,bytes32)"
    )]
    pub struct UpdateDefaultAggchainVKeyFilter {
        pub selector: [u8; 4],
        pub new_v_key: [u8; 32],
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
        name = "UpdatePessimisticVKey",
        abi = "UpdatePessimisticVKey(bytes4,address,bytes32)"
    )]
    pub struct UpdatePessimisticVKeyFilter {
        pub selector: [u8; 4],
        pub verifier: ::ethers::core::types::Address,
        pub new_pessimistic_v_key: [u8; 32],
    }
    ///Container type for all of the contract's events
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum AggLayerGatewayEvents {
        AcceptAggLayerAdminRoleFilter(AcceptAggLayerAdminRoleFilter),
        AddDefaultAggchainVKeyFilter(AddDefaultAggchainVKeyFilter),
        InitializedFilter(InitializedFilter),
        RoleAdminChangedFilter(RoleAdminChangedFilter),
        RoleGrantedFilter(RoleGrantedFilter),
        RoleRevokedFilter(RoleRevokedFilter),
        RouteAddedFilter(RouteAddedFilter),
        RouteFrozenFilter(RouteFrozenFilter),
        TransferAggLayerAdminRoleFilter(TransferAggLayerAdminRoleFilter),
        UpdateDefaultAggchainVKeyFilter(UpdateDefaultAggchainVKeyFilter),
        UpdatePessimisticVKeyFilter(UpdatePessimisticVKeyFilter),
    }
    impl ::ethers::contract::EthLogDecode for AggLayerGatewayEvents {
        fn decode_log(
            log: &::ethers::core::abi::RawLog,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::Error> {
            if let Ok(decoded) = AcceptAggLayerAdminRoleFilter::decode_log(log) {
                return Ok(AggLayerGatewayEvents::AcceptAggLayerAdminRoleFilter(decoded));
            }
            if let Ok(decoded) = AddDefaultAggchainVKeyFilter::decode_log(log) {
                return Ok(AggLayerGatewayEvents::AddDefaultAggchainVKeyFilter(decoded));
            }
            if let Ok(decoded) = InitializedFilter::decode_log(log) {
                return Ok(AggLayerGatewayEvents::InitializedFilter(decoded));
            }
            if let Ok(decoded) = RoleAdminChangedFilter::decode_log(log) {
                return Ok(AggLayerGatewayEvents::RoleAdminChangedFilter(decoded));
            }
            if let Ok(decoded) = RoleGrantedFilter::decode_log(log) {
                return Ok(AggLayerGatewayEvents::RoleGrantedFilter(decoded));
            }
            if let Ok(decoded) = RoleRevokedFilter::decode_log(log) {
                return Ok(AggLayerGatewayEvents::RoleRevokedFilter(decoded));
            }
            if let Ok(decoded) = RouteAddedFilter::decode_log(log) {
                return Ok(AggLayerGatewayEvents::RouteAddedFilter(decoded));
            }
            if let Ok(decoded) = RouteFrozenFilter::decode_log(log) {
                return Ok(AggLayerGatewayEvents::RouteFrozenFilter(decoded));
            }
            if let Ok(decoded) = TransferAggLayerAdminRoleFilter::decode_log(log) {
                return Ok(
                    AggLayerGatewayEvents::TransferAggLayerAdminRoleFilter(decoded),
                );
            }
            if let Ok(decoded) = UpdateDefaultAggchainVKeyFilter::decode_log(log) {
                return Ok(
                    AggLayerGatewayEvents::UpdateDefaultAggchainVKeyFilter(decoded),
                );
            }
            if let Ok(decoded) = UpdatePessimisticVKeyFilter::decode_log(log) {
                return Ok(AggLayerGatewayEvents::UpdatePessimisticVKeyFilter(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::core::fmt::Display for AggLayerGatewayEvents {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::AcceptAggLayerAdminRoleFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::AddDefaultAggchainVKeyFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InitializedFilter(element) => ::core::fmt::Display::fmt(element, f),
                Self::RoleAdminChangedFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RoleGrantedFilter(element) => ::core::fmt::Display::fmt(element, f),
                Self::RoleRevokedFilter(element) => ::core::fmt::Display::fmt(element, f),
                Self::RouteAddedFilter(element) => ::core::fmt::Display::fmt(element, f),
                Self::RouteFrozenFilter(element) => ::core::fmt::Display::fmt(element, f),
                Self::TransferAggLayerAdminRoleFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::UpdateDefaultAggchainVKeyFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::UpdatePessimisticVKeyFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
            }
        }
    }
    impl ::core::convert::From<AcceptAggLayerAdminRoleFilter> for AggLayerGatewayEvents {
        fn from(value: AcceptAggLayerAdminRoleFilter) -> Self {
            Self::AcceptAggLayerAdminRoleFilter(value)
        }
    }
    impl ::core::convert::From<AddDefaultAggchainVKeyFilter> for AggLayerGatewayEvents {
        fn from(value: AddDefaultAggchainVKeyFilter) -> Self {
            Self::AddDefaultAggchainVKeyFilter(value)
        }
    }
    impl ::core::convert::From<InitializedFilter> for AggLayerGatewayEvents {
        fn from(value: InitializedFilter) -> Self {
            Self::InitializedFilter(value)
        }
    }
    impl ::core::convert::From<RoleAdminChangedFilter> for AggLayerGatewayEvents {
        fn from(value: RoleAdminChangedFilter) -> Self {
            Self::RoleAdminChangedFilter(value)
        }
    }
    impl ::core::convert::From<RoleGrantedFilter> for AggLayerGatewayEvents {
        fn from(value: RoleGrantedFilter) -> Self {
            Self::RoleGrantedFilter(value)
        }
    }
    impl ::core::convert::From<RoleRevokedFilter> for AggLayerGatewayEvents {
        fn from(value: RoleRevokedFilter) -> Self {
            Self::RoleRevokedFilter(value)
        }
    }
    impl ::core::convert::From<RouteAddedFilter> for AggLayerGatewayEvents {
        fn from(value: RouteAddedFilter) -> Self {
            Self::RouteAddedFilter(value)
        }
    }
    impl ::core::convert::From<RouteFrozenFilter> for AggLayerGatewayEvents {
        fn from(value: RouteFrozenFilter) -> Self {
            Self::RouteFrozenFilter(value)
        }
    }
    impl ::core::convert::From<TransferAggLayerAdminRoleFilter>
    for AggLayerGatewayEvents {
        fn from(value: TransferAggLayerAdminRoleFilter) -> Self {
            Self::TransferAggLayerAdminRoleFilter(value)
        }
    }
    impl ::core::convert::From<UpdateDefaultAggchainVKeyFilter>
    for AggLayerGatewayEvents {
        fn from(value: UpdateDefaultAggchainVKeyFilter) -> Self {
            Self::UpdateDefaultAggchainVKeyFilter(value)
        }
    }
    impl ::core::convert::From<UpdatePessimisticVKeyFilter> for AggLayerGatewayEvents {
        fn from(value: UpdatePessimisticVKeyFilter) -> Self {
            Self::UpdatePessimisticVKeyFilter(value)
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
    ///Container type for all input parameters for the `addDefaultAggchainVKey` function with signature `addDefaultAggchainVKey(bytes4,bytes32)` and selector `0x818c8c21`
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
        name = "addDefaultAggchainVKey",
        abi = "addDefaultAggchainVKey(bytes4,bytes32)"
    )]
    pub struct AddDefaultAggchainVKeyCall {
        pub default_aggchain_selector: [u8; 4],
        pub new_aggchain_v_key: [u8; 32],
    }
    ///Container type for all input parameters for the `addPessimisticVKeyRoute` function with signature `addPessimisticVKeyRoute(bytes4,address,bytes32)` and selector `0x9584a516`
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
        name = "addPessimisticVKeyRoute",
        abi = "addPessimisticVKeyRoute(bytes4,address,bytes32)"
    )]
    pub struct AddPessimisticVKeyRouteCall {
        pub pessimistic_v_key_selector: [u8; 4],
        pub verifier: ::ethers::core::types::Address,
        pub pessimistic_v_key: [u8; 32],
    }
    ///Container type for all input parameters for the `defaultAggchainVKeys` function with signature `defaultAggchainVKeys(bytes4)` and selector `0x82bfaea1`
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
    #[ethcall(name = "defaultAggchainVKeys", abi = "defaultAggchainVKeys(bytes4)")]
    pub struct DefaultAggchainVKeysCall {
        pub default_aggchain_selector: [u8; 4],
    }
    ///Container type for all input parameters for the `freezePessimisticVKeyRoute` function with signature `freezePessimisticVKeyRoute(bytes4)` and selector `0x95d2a391`
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
        name = "freezePessimisticVKeyRoute",
        abi = "freezePessimisticVKeyRoute(bytes4)"
    )]
    pub struct FreezePessimisticVKeyRouteCall {
        pub pessimistic_v_key_selector: [u8; 4],
    }
    ///Container type for all input parameters for the `getDefaultAggchainVKey` function with signature `getDefaultAggchainVKey(bytes4)` and selector `0x6cabfdab`
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
    #[ethcall(name = "getDefaultAggchainVKey", abi = "getDefaultAggchainVKey(bytes4)")]
    pub struct GetDefaultAggchainVKeyCall {
        pub default_aggchain_selector: [u8; 4],
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
    ///Container type for all input parameters for the `initialize` function with signature `initialize(address,address,address,address)` and selector `0xf8c8765e`
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
    #[ethcall(name = "initialize", abi = "initialize(address,address,address,address)")]
    pub struct InitializeCall {
        pub default_admin: ::ethers::core::types::Address,
        pub aggchain_default_v_key_role: ::ethers::core::types::Address,
        pub add_route_role: ::ethers::core::types::Address,
        pub freeze_route_role: ::ethers::core::types::Address,
    }
    ///Container type for all input parameters for the `pessimisticVKeyRoutes` function with signature `pessimisticVKeyRoutes(bytes4)` and selector `0x716be075`
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
    #[ethcall(name = "pessimisticVKeyRoutes", abi = "pessimisticVKeyRoutes(bytes4)")]
    pub struct PessimisticVKeyRoutesCall {
        pub pessimistic_v_key_selector: [u8; 4],
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
        pub caller_confirmation: ::ethers::core::types::Address,
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
    ///Container type for all input parameters for the `supportsInterface` function with signature `supportsInterface(bytes4)` and selector `0x01ffc9a7`
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
    #[ethcall(name = "supportsInterface", abi = "supportsInterface(bytes4)")]
    pub struct SupportsInterfaceCall {
        pub interface_id: [u8; 4],
    }
    ///Container type for all input parameters for the `updateDefaultAggchainVKey` function with signature `updateDefaultAggchainVKey(bytes4,bytes32)` and selector `0xf4c44681`
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
        name = "updateDefaultAggchainVKey",
        abi = "updateDefaultAggchainVKey(bytes4,bytes32)"
    )]
    pub struct UpdateDefaultAggchainVKeyCall {
        pub default_aggchain_selector: [u8; 4],
        pub new_default_aggchain_v_key: [u8; 32],
    }
    ///Container type for all input parameters for the `verifyPessimisticProof` function with signature `verifyPessimisticProof(bytes,bytes)` and selector `0xa48fd377`
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
        name = "verifyPessimisticProof",
        abi = "verifyPessimisticProof(bytes,bytes)"
    )]
    pub struct VerifyPessimisticProofCall {
        pub public_values: ::ethers::core::types::Bytes,
        pub proof_bytes: ::ethers::core::types::Bytes,
    }
    ///Container type for all of the contract's call
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum AggLayerGatewayCalls {
        DefaultAdminRole(DefaultAdminRoleCall),
        AddDefaultAggchainVKey(AddDefaultAggchainVKeyCall),
        AddPessimisticVKeyRoute(AddPessimisticVKeyRouteCall),
        DefaultAggchainVKeys(DefaultAggchainVKeysCall),
        FreezePessimisticVKeyRoute(FreezePessimisticVKeyRouteCall),
        GetDefaultAggchainVKey(GetDefaultAggchainVKeyCall),
        GetRoleAdmin(GetRoleAdminCall),
        GrantRole(GrantRoleCall),
        HasRole(HasRoleCall),
        Initialize(InitializeCall),
        PessimisticVKeyRoutes(PessimisticVKeyRoutesCall),
        RenounceRole(RenounceRoleCall),
        RevokeRole(RevokeRoleCall),
        SupportsInterface(SupportsInterfaceCall),
        UpdateDefaultAggchainVKey(UpdateDefaultAggchainVKeyCall),
        VerifyPessimisticProof(VerifyPessimisticProofCall),
    }
    impl ::ethers::core::abi::AbiDecode for AggLayerGatewayCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded) = <DefaultAdminRoleCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::DefaultAdminRole(decoded));
            }
            if let Ok(decoded) = <AddDefaultAggchainVKeyCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::AddDefaultAggchainVKey(decoded));
            }
            if let Ok(decoded) = <AddPessimisticVKeyRouteCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::AddPessimisticVKeyRoute(decoded));
            }
            if let Ok(decoded) = <DefaultAggchainVKeysCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::DefaultAggchainVKeys(decoded));
            }
            if let Ok(decoded) = <FreezePessimisticVKeyRouteCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::FreezePessimisticVKeyRoute(decoded));
            }
            if let Ok(decoded) = <GetDefaultAggchainVKeyCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GetDefaultAggchainVKey(decoded));
            }
            if let Ok(decoded) = <GetRoleAdminCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GetRoleAdmin(decoded));
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
            if let Ok(decoded) = <PessimisticVKeyRoutesCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::PessimisticVKeyRoutes(decoded));
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
            if let Ok(decoded) = <SupportsInterfaceCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::SupportsInterface(decoded));
            }
            if let Ok(decoded) = <UpdateDefaultAggchainVKeyCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::UpdateDefaultAggchainVKey(decoded));
            }
            if let Ok(decoded) = <VerifyPessimisticProofCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::VerifyPessimisticProof(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for AggLayerGatewayCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                Self::DefaultAdminRole(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::AddDefaultAggchainVKey(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::AddPessimisticVKeyRoute(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::DefaultAggchainVKeys(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::FreezePessimisticVKeyRoute(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetDefaultAggchainVKey(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetRoleAdmin(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GrantRole(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::HasRole(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::Initialize(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::PessimisticVKeyRoutes(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RenounceRole(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RevokeRole(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::SupportsInterface(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::UpdateDefaultAggchainVKey(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::VerifyPessimisticProof(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
            }
        }
    }
    impl ::core::fmt::Display for AggLayerGatewayCalls {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::DefaultAdminRole(element) => ::core::fmt::Display::fmt(element, f),
                Self::AddDefaultAggchainVKey(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::AddPessimisticVKeyRoute(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::DefaultAggchainVKeys(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::FreezePessimisticVKeyRoute(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::GetDefaultAggchainVKey(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::GetRoleAdmin(element) => ::core::fmt::Display::fmt(element, f),
                Self::GrantRole(element) => ::core::fmt::Display::fmt(element, f),
                Self::HasRole(element) => ::core::fmt::Display::fmt(element, f),
                Self::Initialize(element) => ::core::fmt::Display::fmt(element, f),
                Self::PessimisticVKeyRoutes(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RenounceRole(element) => ::core::fmt::Display::fmt(element, f),
                Self::RevokeRole(element) => ::core::fmt::Display::fmt(element, f),
                Self::SupportsInterface(element) => ::core::fmt::Display::fmt(element, f),
                Self::UpdateDefaultAggchainVKey(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::VerifyPessimisticProof(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
            }
        }
    }
    impl ::core::convert::From<DefaultAdminRoleCall> for AggLayerGatewayCalls {
        fn from(value: DefaultAdminRoleCall) -> Self {
            Self::DefaultAdminRole(value)
        }
    }
    impl ::core::convert::From<AddDefaultAggchainVKeyCall> for AggLayerGatewayCalls {
        fn from(value: AddDefaultAggchainVKeyCall) -> Self {
            Self::AddDefaultAggchainVKey(value)
        }
    }
    impl ::core::convert::From<AddPessimisticVKeyRouteCall> for AggLayerGatewayCalls {
        fn from(value: AddPessimisticVKeyRouteCall) -> Self {
            Self::AddPessimisticVKeyRoute(value)
        }
    }
    impl ::core::convert::From<DefaultAggchainVKeysCall> for AggLayerGatewayCalls {
        fn from(value: DefaultAggchainVKeysCall) -> Self {
            Self::DefaultAggchainVKeys(value)
        }
    }
    impl ::core::convert::From<FreezePessimisticVKeyRouteCall> for AggLayerGatewayCalls {
        fn from(value: FreezePessimisticVKeyRouteCall) -> Self {
            Self::FreezePessimisticVKeyRoute(value)
        }
    }
    impl ::core::convert::From<GetDefaultAggchainVKeyCall> for AggLayerGatewayCalls {
        fn from(value: GetDefaultAggchainVKeyCall) -> Self {
            Self::GetDefaultAggchainVKey(value)
        }
    }
    impl ::core::convert::From<GetRoleAdminCall> for AggLayerGatewayCalls {
        fn from(value: GetRoleAdminCall) -> Self {
            Self::GetRoleAdmin(value)
        }
    }
    impl ::core::convert::From<GrantRoleCall> for AggLayerGatewayCalls {
        fn from(value: GrantRoleCall) -> Self {
            Self::GrantRole(value)
        }
    }
    impl ::core::convert::From<HasRoleCall> for AggLayerGatewayCalls {
        fn from(value: HasRoleCall) -> Self {
            Self::HasRole(value)
        }
    }
    impl ::core::convert::From<InitializeCall> for AggLayerGatewayCalls {
        fn from(value: InitializeCall) -> Self {
            Self::Initialize(value)
        }
    }
    impl ::core::convert::From<PessimisticVKeyRoutesCall> for AggLayerGatewayCalls {
        fn from(value: PessimisticVKeyRoutesCall) -> Self {
            Self::PessimisticVKeyRoutes(value)
        }
    }
    impl ::core::convert::From<RenounceRoleCall> for AggLayerGatewayCalls {
        fn from(value: RenounceRoleCall) -> Self {
            Self::RenounceRole(value)
        }
    }
    impl ::core::convert::From<RevokeRoleCall> for AggLayerGatewayCalls {
        fn from(value: RevokeRoleCall) -> Self {
            Self::RevokeRole(value)
        }
    }
    impl ::core::convert::From<SupportsInterfaceCall> for AggLayerGatewayCalls {
        fn from(value: SupportsInterfaceCall) -> Self {
            Self::SupportsInterface(value)
        }
    }
    impl ::core::convert::From<UpdateDefaultAggchainVKeyCall> for AggLayerGatewayCalls {
        fn from(value: UpdateDefaultAggchainVKeyCall) -> Self {
            Self::UpdateDefaultAggchainVKey(value)
        }
    }
    impl ::core::convert::From<VerifyPessimisticProofCall> for AggLayerGatewayCalls {
        fn from(value: VerifyPessimisticProofCall) -> Self {
            Self::VerifyPessimisticProof(value)
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
    ///Container type for all return fields from the `defaultAggchainVKeys` function with signature `defaultAggchainVKeys(bytes4)` and selector `0x82bfaea1`
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
    pub struct DefaultAggchainVKeysReturn {
        pub default_aggchain_v_key: [u8; 32],
    }
    ///Container type for all return fields from the `getDefaultAggchainVKey` function with signature `getDefaultAggchainVKey(bytes4)` and selector `0x6cabfdab`
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
    pub struct GetDefaultAggchainVKeyReturn(pub [u8; 32]);
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
    ///Container type for all return fields from the `pessimisticVKeyRoutes` function with signature `pessimisticVKeyRoutes(bytes4)` and selector `0x716be075`
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
    pub struct PessimisticVKeyRoutesReturn {
        pub verifier: ::ethers::core::types::Address,
        pub pessimistic_v_key: [u8; 32],
        pub frozen: bool,
    }
    ///Container type for all return fields from the `supportsInterface` function with signature `supportsInterface(bytes4)` and selector `0x01ffc9a7`
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
    pub struct SupportsInterfaceReturn(pub bool);
}
