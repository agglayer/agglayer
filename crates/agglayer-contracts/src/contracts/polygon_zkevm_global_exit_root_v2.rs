pub use polygon_zk_evm_global_exit_root_v2::*;
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
pub mod polygon_zk_evm_global_exit_root_v2 {
    #[allow(deprecated)]
    fn __abi() -> ::ethers::core::abi::Abi {
        ::ethers::core::abi::ethabi::Contract {
            constructor: ::core::option::Option::Some(::ethers::core::abi::ethabi::Constructor {
                inputs: ::std::vec![
                    ::ethers::core::abi::ethabi::Param {
                        name: ::std::borrow::ToOwned::to_owned("_rollupManager"),
                        kind: ::ethers::core::abi::ethabi::ParamType::Address,
                        internal_type: ::core::option::Option::Some(
                            ::std::borrow::ToOwned::to_owned("address"),
                        ),
                    },
                    ::ethers::core::abi::ethabi::Param {
                        name: ::std::borrow::ToOwned::to_owned("_bridgeAddress"),
                        kind: ::ethers::core::abi::ethabi::ParamType::Address,
                        internal_type: ::core::option::Option::Some(
                            ::std::borrow::ToOwned::to_owned("address"),
                        ),
                    },
                ],
            }),
            functions: ::core::convert::From::from([
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
                    ::std::borrow::ToOwned::to_owned("calculateRoot"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("calculateRoot"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("leafHash"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("smtProof"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedArray(
                                        ::std::boxed::Box::new(
                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                        ),
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32[32]"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("index"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint32"),
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
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::Pure,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("depositCount"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("depositCount"),
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
                    ::std::borrow::ToOwned::to_owned("getLastGlobalExitRoot"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "getLastGlobalExitRoot",
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
                    ::std::borrow::ToOwned::to_owned("getLeafValue"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("getLeafValue"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("newGlobalExitRoot"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("lastBlockHash"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("timestamp"),
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
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::Pure,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getRoot"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("getRoot"),
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
                    ::std::borrow::ToOwned::to_owned("globalExitRootMap"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("globalExitRootMap"),
                            inputs: ::std::vec![
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
                    ::std::borrow::ToOwned::to_owned("l1InfoRootMap"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("l1InfoRootMap"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("leafCount"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint32"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("l1InfoRoot"),
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
                    ::std::borrow::ToOwned::to_owned("lastMainnetExitRoot"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "lastMainnetExitRoot",
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
                    ::std::borrow::ToOwned::to_owned("lastRollupExitRoot"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("lastRollupExitRoot"),
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
                    ::std::borrow::ToOwned::to_owned("updateExitRoot"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("updateExitRoot"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("newRoot"),
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
                    ::std::borrow::ToOwned::to_owned("verifyMerkleProof"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("verifyMerkleProof"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("leafHash"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("smtProof"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedArray(
                                        ::std::boxed::Box::new(
                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                        ),
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32[32]"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("index"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("root"),
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
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bool"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::Pure,
                        },
                    ],
                ),
            ]),
            events: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("InitL1InfoRootMap"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("InitL1InfoRootMap"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("leafCount"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("currentL1InfoRoot"),
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
                    ::std::borrow::ToOwned::to_owned("UpdateL1InfoTree"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("UpdateL1InfoTree"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("mainnetExitRoot"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    indexed: true,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("rollupExitRoot"),
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
                    ::std::borrow::ToOwned::to_owned("UpdateL1InfoTreeV2"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("UpdateL1InfoTreeV2"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("currentL1InfoRoot"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("leafCount"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(32usize),
                                    indexed: true,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("blockhash"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("minTimestamp"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
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
                    ::std::borrow::ToOwned::to_owned("GlobalExitRootAlreadySet"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "GlobalExitRootAlreadySet",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("GlobalExitRootNotFound"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "GlobalExitRootNotFound",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("MerkleTreeFull"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("MerkleTreeFull"),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("OnlyAllowedContracts"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "OnlyAllowedContracts",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("OnlyGlobalExitRootRemover"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "OnlyGlobalExitRootRemover",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("OnlyGlobalExitRootUpdater"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "OnlyGlobalExitRootUpdater",
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
    pub static POLYGONZKEVMGLOBALEXITROOTV2_ABI: ::ethers::contract::Lazy<
        ::ethers::core::abi::Abi,
    > = ::ethers::contract::Lazy::new(__abi);
    #[rustfmt::skip]
    const __BYTECODE: &[u8] = b"`\xC0`@R4\x80\x15a\0\x0FW__\xFD[P`@Qa\x0E\xC08\x03\x80a\x0E\xC0\x839\x81\x01`@\x81\x90Ra\0.\x91a\x01+V[`\x01`\x01`\xA0\x1B\x03\x80\x83\x16`\xA0R\x81\x16`\x80Ra\0Ia\0PV[PPa\x01\\V[`.Ta\x01\0\x90\x04`\xFF\x16\x15a\0\xBCW`@QbF\x1B\xCD`\xE5\x1B\x81R` `\x04\x82\x01R`'`$\x82\x01R\x7FInitializable: contract is initi`D\x82\x01Rfalizing`\xC8\x1B`d\x82\x01R`\x84\x01`@Q\x80\x91\x03\x90\xFD[`.T`\xFF\x90\x81\x16\x10\x15a\x01\x0EW`.\x80T`\xFF\x19\x16`\xFF\x90\x81\x17\x90\x91U`@Q\x90\x81R\x7F\x7F&\xB8?\xF9n\x1F+jh/\x138R\xF6y\x8A\t\xC4e\xDA\x95\x92\x14`\xCE\xFB8G@$\x98\x90` \x01`@Q\x80\x91\x03\x90\xA1[V[\x80Q`\x01`\x01`\xA0\x1B\x03\x81\x16\x81\x14a\x01&W__\xFD[\x91\x90PV[__`@\x83\x85\x03\x12\x15a\x01<W__\xFD[a\x01E\x83a\x01\x10V[\x91Pa\x01S` \x84\x01a\x01\x10V[\x90P\x92P\x92\x90PV[`\x80Q`\xA0Qa\r5a\x01\x8B_9_\x81\x81a\x01W\x01Ra\x02\xF7\x01R_\x81\x81a\x02.\x01Ra\x02\xAB\x01Ra\r5_\xF3\xFE`\x80`@R4\x80\x15a\0\x0FW__\xFD[P`\x046\x10a\0\xE5W_5`\xE0\x1C\x80c\\\xA1\xE1e\x11a\0\x88W\x80c\x83\xF2D\x03\x11a\0cW\x80c\x83\xF2D\x03\x14a\x02\x16W\x80c\xA3\xC5s\xEB\x14a\x02)W\x80c\xEFN\xEB5\x14a\x02PW\x80c\xFBW\x084\x14a\x02oW__\xFD[\x80c\\\xA1\xE1e\x14a\x01\x9EW\x80c]\x81\x05\x01\x14a\x01\xA6W\x80c\x81)\xFC\x1C\x14a\x02\x0EW__\xFD[\x80c1\x9C\xF75\x11a\0\xC3W\x80c1\x9C\xF75\x14a\x01,W\x80c3\xD6$}\x14a\x015W\x80c>\xD6\x91\xEF\x14a\x01JW\x80cI\xB7\xB8\x02\x14a\x01RW__\xFD[\x80c\x01\xFD\x90D\x14a\0\xE9W\x80c%{62\x14a\x01\x04W\x80c-\xFD\xF0\xB5\x14a\x01#W[__\xFD[a\0\xF1_T\x81V[`@Q\x90\x81R` \x01[`@Q\x80\x91\x03\x90\xF3[a\0\xF1a\x01\x126`\x04a\t\xDFV[`\x02` R_\x90\x81R`@\x90 T\x81V[a\0\xF1`#T\x81V[a\0\xF1`\x01T\x81V[a\x01Ha\x01C6`\x04a\t\xDFV[a\x02\x92V[\0[a\0\xF1a\x04\xB8V[a\x01y\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81V[`@Qs\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x90\x91\x16\x81R` \x01a\0\xFBV[a\0\xF1a\x04\xCBV[a\0\xF1a\x01\xB46`\x04a\t\xF6V[`@\x80Q` \x80\x82\x01\x95\x90\x95R\x80\x82\x01\x93\x90\x93R`\xC0\x91\x90\x91\x1B\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x16``\x83\x01R\x80Q`H\x81\x84\x03\x01\x81R`h\x90\x92\x01\x90R\x80Q\x91\x01 \x90V[a\x01Ha\x04\xD4V[a\0\xF1a\x02$6`\x04a\naV[a\x06\xC8V[a\x01y\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81V[a\0\xF1a\x02^6`\x04a\n\x9DV[`/` R_\x90\x81R`@\x90 T\x81V[a\x02\x82a\x02}6`\x04a\n\xBDV[a\x07\x93V[`@Q\x90\x15\x15\x81R` \x01a\0\xFBV[_\x80s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x163\x03a\x02\xE0WPP`\x01\x81\x90U_T\x81a\x03_V[s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x163\x03a\x03-WPP_\x81\x90U`\x01T\x81\x90a\x03_V[`@Q\x7F\xB4\x93e\xDD\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[_a\x03j\x82\x84a\x07\xAAV[_\x81\x81R`\x02` R`@\x81 T\x91\x92P\x03a\x04\xB2WB_a\x03\x8D`\x01Ca\x0B/V[_\x84\x81R`\x02` \x90\x81R`@\x91\x82\x90 \x92@\x92\x83\x90U\x81Q\x80\x82\x01\x87\x90R\x80\x83\x01\x84\x90R\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`\xC0\x87\x90\x1B\x16``\x82\x01R\x82Q\x80\x82\x03`H\x01\x81R`h\x90\x91\x01\x90\x92R\x81Q\x91\x01 \x90\x91Pa\x04\x01\x90a\x07\xD9V[_a\x04\na\x04\xCBV[`#Tc\xFF\xFF\xFF\xFF\x16_\x90\x81R`/` R`@\x80\x82 \x83\x90UQ\x91\x92P\x87\x91\x87\x91\x7F\xDAa\xAAx#\xFC\xD8\x07\xE3{\x95\xAA\xBC\xBE\x17\xF0:o>\xFDQAvDM\xAE\x19\x1D'\xFDf\xB3\x91\xA3`#Tc\xFF\xFF\xFF\xFF\x16\x7F\xAFll\xD7y\x0E\x01\x80\xA4\xD2.\xB8\xED\x84nU\x84oT\xED\x10\xE5\x94m\xB1\x99r\xB5\xA0\x81:Y\x82\x84\x86`@Qa\x04\xA6\x93\x92\x91\x90\x92\x83R` \x83\x01\x91\x90\x91Rg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x16`@\x82\x01R``\x01\x90V[`@Q\x80\x91\x03\x90\xA2PPP[PPPPV[_a\x04\xC6`\x01T_Ta\x07\xAAV[\x90P\x90V[_a\x04\xC6a\x08\xEEV[`.Ta\x01\0\x90\x04`\xFF\x16\x15\x80\x80\x15a\x04\xF4WP`.T`\x01`\xFF\x90\x91\x16\x10[\x80a\x05\x0EWP0;\x15\x80\x15a\x05\x0EWP`.T`\xFF\x16`\x01\x14[a\x05\x9EW`@Q\x7F\x08\xC3y\xA0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R` `\x04\x82\x01R`.`$\x82\x01R\x7FInitializable: contract is alrea`D\x82\x01R\x7Fdy initialized\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`d\x82\x01R`\x84\x01`@Q\x80\x91\x03\x90\xFD[`.\x80T\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\0\x16`\x01\x17\x90U\x80\x15a\x05\xFCW`.\x80T\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\0\xFF\x16a\x01\0\x17\x90U[_a\x06\x05a\x04\xCBV[`#\x80Tc\xFF\xFF\xFF\xFF\x90\x81\x16_\x90\x81R`/` \x90\x81R`@\x91\x82\x90 \x85\x90U\x92T\x81Q\x92\x16\x82R\x91\x81\x01\x83\x90R\x91\x92P\x7F\x11\xF5\x0Cq\x89\x10\x02\x83\x9C&7\xCE0 \x87\x16\x02\x98%Z\x87\xF1\xEA`\xD4\x0E\x8D\xB0\x818?\xAD\x91\x01`@Q\x80\x91\x03\x90\xA1P\x80\x15a\x06\xC5W`.\x80T\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\0\xFF\x16\x90U`@Q`\x01\x81R\x7F\x7F&\xB8?\xF9n\x1F+jh/\x138R\xF6y\x8A\t\xC4e\xDA\x95\x92\x14`\xCE\xFB8G@$\x98\x90` \x01`@Q\x80\x91\x03\x90\xA1[PV[_\x83\x81[` \x81\x10\x15a\x07\x8AW`\x01c\xFF\xFF\xFF\xFF\x85\x16\x82\x1C\x81\x16\x90\x03a\x077W\x84\x81` \x81\x10a\x06\xFAWa\x06\xFAa\x0BBV[` \x02\x015\x82`@Q` \x01a\x07\x1A\x92\x91\x90\x91\x82R` \x82\x01R`@\x01\x90V[`@Q` \x81\x83\x03\x03\x81R\x90`@R\x80Q\x90` \x01 \x91Pa\x07\x82V[\x81\x85\x82` \x81\x10a\x07JWa\x07Ja\x0BBV[` \x02\x015`@Q` \x01a\x07i\x92\x91\x90\x91\x82R` \x82\x01R`@\x01\x90V[`@Q` \x81\x83\x03\x03\x81R\x90`@R\x80Q\x90` \x01 \x91P[`\x01\x01a\x06\xCCV[P\x94\x93PPPPV[_\x81a\x07\xA0\x86\x86\x86a\x06\xC8V[\x14\x95\x94PPPPPV[`@\x80Q` \x80\x82\x01\x85\x90R\x81\x83\x01\x84\x90R\x82Q\x80\x83\x03\x84\x01\x81R``\x90\x92\x01\x90\x92R\x80Q\x91\x01 [\x92\x91PPV[\x80`\x01a\x07\xE8` `\x02a\x0C\x90V[a\x07\xF2\x91\x90a\x0B/V[`#T\x10a\x08,W`@Q\x7F\xEF\\\xCFf\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[_`#_\x81Ta\x08;\x90a\x0C\x9BV[\x91\x82\x90UP\x90P_[` \x81\x10\x15a\x08\xE0W\x80\x82\x90\x1C`\x01\x16`\x01\x03a\x08wW\x82`\x03\x82` \x81\x10a\x08oWa\x08oa\x0BBV[\x01UPPPPV[`\x03\x81` \x81\x10a\x08\x8AWa\x08\x8Aa\x0BBV[\x01T`@\x80Q` \x81\x01\x92\x90\x92R\x81\x01\x84\x90R``\x01`@\x80Q\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xE0\x81\x84\x03\x01\x81R\x91\x90R\x80Q` \x90\x91\x01 \x92P`\x01\x01a\x08DV[Pa\x08\xE9a\x0C\xD2V[PPPV[`#T_\x90\x81\x90\x81\x80[` \x81\x10\x15a\t\xD6W\x80\x83\x90\x1C`\x01\x16`\x01\x03a\tUW`\x03\x81` \x81\x10a\t\"Wa\t\"a\x0BBV[\x01T`@\x80Q` \x81\x01\x92\x90\x92R\x81\x01\x85\x90R``\x01`@Q` \x81\x83\x03\x03\x81R\x90`@R\x80Q\x90` \x01 \x93Pa\t\x82V[`@\x80Q` \x81\x01\x86\x90R\x90\x81\x01\x83\x90R``\x01`@Q` \x81\x83\x03\x03\x81R\x90`@R\x80Q\x90` \x01 \x93P[`@\x80Q` \x81\x01\x84\x90R\x90\x81\x01\x83\x90R``\x01`@\x80Q\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xE0\x81\x84\x03\x01\x81R\x91\x90R\x80Q` \x90\x91\x01 \x91P`\x01\x01a\x08\xF8V[P\x91\x93\x92PPPV[_` \x82\x84\x03\x12\x15a\t\xEFW__\xFD[P5\x91\x90PV[___``\x84\x86\x03\x12\x15a\n\x08W__\xFD[\x835\x92P` \x84\x015\x91P`@\x84\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x16\x81\x14a\n-W__\xFD[\x80\x91PP\x92P\x92P\x92V[\x80a\x04\0\x81\x01\x83\x10\x15a\x07\xD3W__\xFD[\x805c\xFF\xFF\xFF\xFF\x81\x16\x81\x14a\n\\W__\xFD[\x91\x90PV[___a\x04@\x84\x86\x03\x12\x15a\ntW__\xFD[\x835\x92Pa\n\x85\x85` \x86\x01a\n8V[\x91Pa\n\x94a\x04 \x85\x01a\nIV[\x90P\x92P\x92P\x92V[_` \x82\x84\x03\x12\x15a\n\xADW__\xFD[a\n\xB6\x82a\nIV[\x93\x92PPPV[____a\x04`\x85\x87\x03\x12\x15a\n\xD1W__\xFD[\x845\x93Pa\n\xE2\x86` \x87\x01a\n8V[\x92Pa\n\xF1a\x04 \x86\x01a\nIV[\x93\x96\x92\x95P\x92\x93a\x04@\x015\x92PPV[\x7FNH{q\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0_R`\x11`\x04R`$_\xFD[\x81\x81\x03\x81\x81\x11\x15a\x07\xD3Wa\x07\xD3a\x0B\x02V[\x7FNH{q\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0_R`2`\x04R`$_\xFD[`\x01\x81[`\x01\x84\x11\x15a\x0B\xAAW\x80\x85\x04\x81\x11\x15a\x0B\x8EWa\x0B\x8Ea\x0B\x02V[`\x01\x84\x16\x15a\x0B\x9CW\x90\x81\x02\x90[`\x01\x93\x90\x93\x1C\x92\x80\x02a\x0BsV[\x93P\x93\x91PPV[_\x82a\x0B\xC0WP`\x01a\x07\xD3V[\x81a\x0B\xCCWP_a\x07\xD3V[\x81`\x01\x81\x14a\x0B\xE2W`\x02\x81\x14a\x0B\xECWa\x0C\x08V[`\x01\x91PPa\x07\xD3V[`\xFF\x84\x11\x15a\x0B\xFDWa\x0B\xFDa\x0B\x02V[PP`\x01\x82\x1Ba\x07\xD3V[P` \x83\x10a\x013\x83\x10\x16`N\x84\x10`\x0B\x84\x10\x16\x17\x15a\x0C+WP\x81\x81\na\x07\xD3V[a\x0CV\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x84\x84a\x0BoV[\x80\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x04\x82\x11\x15a\x0C\x88Wa\x0C\x88a\x0B\x02V[\x02\x93\x92PPPV[_a\n\xB6\x83\x83a\x0B\xB2V[_\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x03a\x0C\xCBWa\x0C\xCBa\x0B\x02V[P`\x01\x01\x90V[\x7FNH{q\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0_R`\x01`\x04R`$_\xFD\xFE\xA2dipfsX\"\x12 \x94M\x82v&\xA71\x07@Hx\x95\x1EtP\xE9?\xA7&\xED\xCB4\xF8B\xD4\x93\x88\x03e\x16\x1D\x07dsolcC\0\x08\x1C\x003";
    /// The bytecode of the contract.
    pub static POLYGONZKEVMGLOBALEXITROOTV2_BYTECODE: ::ethers::core::types::Bytes = ::ethers::core::types::Bytes::from_static(
        __BYTECODE,
    );
    #[rustfmt::skip]
    const __DEPLOYED_BYTECODE: &[u8] = b"`\x80`@R4\x80\x15a\0\x0FW__\xFD[P`\x046\x10a\0\xE5W_5`\xE0\x1C\x80c\\\xA1\xE1e\x11a\0\x88W\x80c\x83\xF2D\x03\x11a\0cW\x80c\x83\xF2D\x03\x14a\x02\x16W\x80c\xA3\xC5s\xEB\x14a\x02)W\x80c\xEFN\xEB5\x14a\x02PW\x80c\xFBW\x084\x14a\x02oW__\xFD[\x80c\\\xA1\xE1e\x14a\x01\x9EW\x80c]\x81\x05\x01\x14a\x01\xA6W\x80c\x81)\xFC\x1C\x14a\x02\x0EW__\xFD[\x80c1\x9C\xF75\x11a\0\xC3W\x80c1\x9C\xF75\x14a\x01,W\x80c3\xD6$}\x14a\x015W\x80c>\xD6\x91\xEF\x14a\x01JW\x80cI\xB7\xB8\x02\x14a\x01RW__\xFD[\x80c\x01\xFD\x90D\x14a\0\xE9W\x80c%{62\x14a\x01\x04W\x80c-\xFD\xF0\xB5\x14a\x01#W[__\xFD[a\0\xF1_T\x81V[`@Q\x90\x81R` \x01[`@Q\x80\x91\x03\x90\xF3[a\0\xF1a\x01\x126`\x04a\t\xDFV[`\x02` R_\x90\x81R`@\x90 T\x81V[a\0\xF1`#T\x81V[a\0\xF1`\x01T\x81V[a\x01Ha\x01C6`\x04a\t\xDFV[a\x02\x92V[\0[a\0\xF1a\x04\xB8V[a\x01y\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81V[`@Qs\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x90\x91\x16\x81R` \x01a\0\xFBV[a\0\xF1a\x04\xCBV[a\0\xF1a\x01\xB46`\x04a\t\xF6V[`@\x80Q` \x80\x82\x01\x95\x90\x95R\x80\x82\x01\x93\x90\x93R`\xC0\x91\x90\x91\x1B\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x16``\x83\x01R\x80Q`H\x81\x84\x03\x01\x81R`h\x90\x92\x01\x90R\x80Q\x91\x01 \x90V[a\x01Ha\x04\xD4V[a\0\xF1a\x02$6`\x04a\naV[a\x06\xC8V[a\x01y\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81V[a\0\xF1a\x02^6`\x04a\n\x9DV[`/` R_\x90\x81R`@\x90 T\x81V[a\x02\x82a\x02}6`\x04a\n\xBDV[a\x07\x93V[`@Q\x90\x15\x15\x81R` \x01a\0\xFBV[_\x80s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x163\x03a\x02\xE0WPP`\x01\x81\x90U_T\x81a\x03_V[s\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x163\x03a\x03-WPP_\x81\x90U`\x01T\x81\x90a\x03_V[`@Q\x7F\xB4\x93e\xDD\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[_a\x03j\x82\x84a\x07\xAAV[_\x81\x81R`\x02` R`@\x81 T\x91\x92P\x03a\x04\xB2WB_a\x03\x8D`\x01Ca\x0B/V[_\x84\x81R`\x02` \x90\x81R`@\x91\x82\x90 \x92@\x92\x83\x90U\x81Q\x80\x82\x01\x87\x90R\x80\x83\x01\x84\x90R\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`\xC0\x87\x90\x1B\x16``\x82\x01R\x82Q\x80\x82\x03`H\x01\x81R`h\x90\x91\x01\x90\x92R\x81Q\x91\x01 \x90\x91Pa\x04\x01\x90a\x07\xD9V[_a\x04\na\x04\xCBV[`#Tc\xFF\xFF\xFF\xFF\x16_\x90\x81R`/` R`@\x80\x82 \x83\x90UQ\x91\x92P\x87\x91\x87\x91\x7F\xDAa\xAAx#\xFC\xD8\x07\xE3{\x95\xAA\xBC\xBE\x17\xF0:o>\xFDQAvDM\xAE\x19\x1D'\xFDf\xB3\x91\xA3`#Tc\xFF\xFF\xFF\xFF\x16\x7F\xAFll\xD7y\x0E\x01\x80\xA4\xD2.\xB8\xED\x84nU\x84oT\xED\x10\xE5\x94m\xB1\x99r\xB5\xA0\x81:Y\x82\x84\x86`@Qa\x04\xA6\x93\x92\x91\x90\x92\x83R` \x83\x01\x91\x90\x91Rg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x16`@\x82\x01R``\x01\x90V[`@Q\x80\x91\x03\x90\xA2PPP[PPPPV[_a\x04\xC6`\x01T_Ta\x07\xAAV[\x90P\x90V[_a\x04\xC6a\x08\xEEV[`.Ta\x01\0\x90\x04`\xFF\x16\x15\x80\x80\x15a\x04\xF4WP`.T`\x01`\xFF\x90\x91\x16\x10[\x80a\x05\x0EWP0;\x15\x80\x15a\x05\x0EWP`.T`\xFF\x16`\x01\x14[a\x05\x9EW`@Q\x7F\x08\xC3y\xA0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R` `\x04\x82\x01R`.`$\x82\x01R\x7FInitializable: contract is alrea`D\x82\x01R\x7Fdy initialized\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0`d\x82\x01R`\x84\x01`@Q\x80\x91\x03\x90\xFD[`.\x80T\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\0\x16`\x01\x17\x90U\x80\x15a\x05\xFCW`.\x80T\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\0\xFF\x16a\x01\0\x17\x90U[_a\x06\x05a\x04\xCBV[`#\x80Tc\xFF\xFF\xFF\xFF\x90\x81\x16_\x90\x81R`/` \x90\x81R`@\x91\x82\x90 \x85\x90U\x92T\x81Q\x92\x16\x82R\x91\x81\x01\x83\x90R\x91\x92P\x7F\x11\xF5\x0Cq\x89\x10\x02\x83\x9C&7\xCE0 \x87\x16\x02\x98%Z\x87\xF1\xEA`\xD4\x0E\x8D\xB0\x818?\xAD\x91\x01`@Q\x80\x91\x03\x90\xA1P\x80\x15a\x06\xC5W`.\x80T\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\0\xFF\x16\x90U`@Q`\x01\x81R\x7F\x7F&\xB8?\xF9n\x1F+jh/\x138R\xF6y\x8A\t\xC4e\xDA\x95\x92\x14`\xCE\xFB8G@$\x98\x90` \x01`@Q\x80\x91\x03\x90\xA1[PV[_\x83\x81[` \x81\x10\x15a\x07\x8AW`\x01c\xFF\xFF\xFF\xFF\x85\x16\x82\x1C\x81\x16\x90\x03a\x077W\x84\x81` \x81\x10a\x06\xFAWa\x06\xFAa\x0BBV[` \x02\x015\x82`@Q` \x01a\x07\x1A\x92\x91\x90\x91\x82R` \x82\x01R`@\x01\x90V[`@Q` \x81\x83\x03\x03\x81R\x90`@R\x80Q\x90` \x01 \x91Pa\x07\x82V[\x81\x85\x82` \x81\x10a\x07JWa\x07Ja\x0BBV[` \x02\x015`@Q` \x01a\x07i\x92\x91\x90\x91\x82R` \x82\x01R`@\x01\x90V[`@Q` \x81\x83\x03\x03\x81R\x90`@R\x80Q\x90` \x01 \x91P[`\x01\x01a\x06\xCCV[P\x94\x93PPPPV[_\x81a\x07\xA0\x86\x86\x86a\x06\xC8V[\x14\x95\x94PPPPPV[`@\x80Q` \x80\x82\x01\x85\x90R\x81\x83\x01\x84\x90R\x82Q\x80\x83\x03\x84\x01\x81R``\x90\x92\x01\x90\x92R\x80Q\x91\x01 [\x92\x91PPV[\x80`\x01a\x07\xE8` `\x02a\x0C\x90V[a\x07\xF2\x91\x90a\x0B/V[`#T\x10a\x08,W`@Q\x7F\xEF\\\xCFf\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x81R`\x04\x01`@Q\x80\x91\x03\x90\xFD[_`#_\x81Ta\x08;\x90a\x0C\x9BV[\x91\x82\x90UP\x90P_[` \x81\x10\x15a\x08\xE0W\x80\x82\x90\x1C`\x01\x16`\x01\x03a\x08wW\x82`\x03\x82` \x81\x10a\x08oWa\x08oa\x0BBV[\x01UPPPPV[`\x03\x81` \x81\x10a\x08\x8AWa\x08\x8Aa\x0BBV[\x01T`@\x80Q` \x81\x01\x92\x90\x92R\x81\x01\x84\x90R``\x01`@\x80Q\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xE0\x81\x84\x03\x01\x81R\x91\x90R\x80Q` \x90\x91\x01 \x92P`\x01\x01a\x08DV[Pa\x08\xE9a\x0C\xD2V[PPPV[`#T_\x90\x81\x90\x81\x80[` \x81\x10\x15a\t\xD6W\x80\x83\x90\x1C`\x01\x16`\x01\x03a\tUW`\x03\x81` \x81\x10a\t\"Wa\t\"a\x0BBV[\x01T`@\x80Q` \x81\x01\x92\x90\x92R\x81\x01\x85\x90R``\x01`@Q` \x81\x83\x03\x03\x81R\x90`@R\x80Q\x90` \x01 \x93Pa\t\x82V[`@\x80Q` \x81\x01\x86\x90R\x90\x81\x01\x83\x90R``\x01`@Q` \x81\x83\x03\x03\x81R\x90`@R\x80Q\x90` \x01 \x93P[`@\x80Q` \x81\x01\x84\x90R\x90\x81\x01\x83\x90R``\x01`@\x80Q\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xE0\x81\x84\x03\x01\x81R\x91\x90R\x80Q` \x90\x91\x01 \x91P`\x01\x01a\x08\xF8V[P\x91\x93\x92PPPV[_` \x82\x84\x03\x12\x15a\t\xEFW__\xFD[P5\x91\x90PV[___``\x84\x86\x03\x12\x15a\n\x08W__\xFD[\x835\x92P` \x84\x015\x91P`@\x84\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x16\x81\x14a\n-W__\xFD[\x80\x91PP\x92P\x92P\x92V[\x80a\x04\0\x81\x01\x83\x10\x15a\x07\xD3W__\xFD[\x805c\xFF\xFF\xFF\xFF\x81\x16\x81\x14a\n\\W__\xFD[\x91\x90PV[___a\x04@\x84\x86\x03\x12\x15a\ntW__\xFD[\x835\x92Pa\n\x85\x85` \x86\x01a\n8V[\x91Pa\n\x94a\x04 \x85\x01a\nIV[\x90P\x92P\x92P\x92V[_` \x82\x84\x03\x12\x15a\n\xADW__\xFD[a\n\xB6\x82a\nIV[\x93\x92PPPV[____a\x04`\x85\x87\x03\x12\x15a\n\xD1W__\xFD[\x845\x93Pa\n\xE2\x86` \x87\x01a\n8V[\x92Pa\n\xF1a\x04 \x86\x01a\nIV[\x93\x96\x92\x95P\x92\x93a\x04@\x015\x92PPV[\x7FNH{q\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0_R`\x11`\x04R`$_\xFD[\x81\x81\x03\x81\x81\x11\x15a\x07\xD3Wa\x07\xD3a\x0B\x02V[\x7FNH{q\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0_R`2`\x04R`$_\xFD[`\x01\x81[`\x01\x84\x11\x15a\x0B\xAAW\x80\x85\x04\x81\x11\x15a\x0B\x8EWa\x0B\x8Ea\x0B\x02V[`\x01\x84\x16\x15a\x0B\x9CW\x90\x81\x02\x90[`\x01\x93\x90\x93\x1C\x92\x80\x02a\x0BsV[\x93P\x93\x91PPV[_\x82a\x0B\xC0WP`\x01a\x07\xD3V[\x81a\x0B\xCCWP_a\x07\xD3V[\x81`\x01\x81\x14a\x0B\xE2W`\x02\x81\x14a\x0B\xECWa\x0C\x08V[`\x01\x91PPa\x07\xD3V[`\xFF\x84\x11\x15a\x0B\xFDWa\x0B\xFDa\x0B\x02V[PP`\x01\x82\x1Ba\x07\xD3V[P` \x83\x10a\x013\x83\x10\x16`N\x84\x10`\x0B\x84\x10\x16\x17\x15a\x0C+WP\x81\x81\na\x07\xD3V[a\x0CV\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x84\x84a\x0BoV[\x80\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x04\x82\x11\x15a\x0C\x88Wa\x0C\x88a\x0B\x02V[\x02\x93\x92PPPV[_a\n\xB6\x83\x83a\x0B\xB2V[_\x7F\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x03a\x0C\xCBWa\x0C\xCBa\x0B\x02V[P`\x01\x01\x90V[\x7FNH{q\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0_R`\x01`\x04R`$_\xFD\xFE\xA2dipfsX\"\x12 \x94M\x82v&\xA71\x07@Hx\x95\x1EtP\xE9?\xA7&\xED\xCB4\xF8B\xD4\x93\x88\x03e\x16\x1D\x07dsolcC\0\x08\x1C\x003";
    /// The deployed bytecode of the contract.
    pub static POLYGONZKEVMGLOBALEXITROOTV2_DEPLOYED_BYTECODE: ::ethers::core::types::Bytes = ::ethers::core::types::Bytes::from_static(
        __DEPLOYED_BYTECODE,
    );
    pub struct PolygonZkEVMGlobalExitRootV2<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for PolygonZkEVMGlobalExitRootV2<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for PolygonZkEVMGlobalExitRootV2<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for PolygonZkEVMGlobalExitRootV2<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for PolygonZkEVMGlobalExitRootV2<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(::core::stringify!(PolygonZkEVMGlobalExitRootV2))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> PolygonZkEVMGlobalExitRootV2<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(
                ::ethers::contract::Contract::new(
                    address.into(),
                    POLYGONZKEVMGLOBALEXITROOTV2_ABI.clone(),
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
                POLYGONZKEVMGLOBALEXITROOTV2_ABI.clone(),
                POLYGONZKEVMGLOBALEXITROOTV2_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ::ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
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
        ///Calls the contract's `calculateRoot` (0x83f24403) function
        pub fn calculate_root(
            &self,
            leaf_hash: [u8; 32],
            smt_proof: [[u8; 32]; 32],
            index: u32,
        ) -> ::ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([131, 242, 68, 3], (leaf_hash, smt_proof, index))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `depositCount` (0x2dfdf0b5) function
        pub fn deposit_count(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([45, 253, 240, 181], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getLastGlobalExitRoot` (0x3ed691ef) function
        pub fn get_last_global_exit_root(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([62, 214, 145, 239], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getLeafValue` (0x5d810501) function
        pub fn get_leaf_value(
            &self,
            new_global_exit_root: [u8; 32],
            last_block_hash: ::ethers::core::types::U256,
            timestamp: u64,
        ) -> ::ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash(
                    [93, 129, 5, 1],
                    (new_global_exit_root, last_block_hash, timestamp),
                )
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getRoot` (0x5ca1e165) function
        pub fn get_root(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([92, 161, 225, 101], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `globalExitRootMap` (0x257b3632) function
        pub fn global_exit_root_map(
            &self,
            p0: [u8; 32],
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([37, 123, 54, 50], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `initialize` (0x8129fc1c) function
        pub fn initialize(&self) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([129, 41, 252, 28], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `l1InfoRootMap` (0xef4eeb35) function
        pub fn l_1_info_root_map(
            &self,
            leaf_count: u32,
        ) -> ::ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([239, 78, 235, 53], leaf_count)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `lastMainnetExitRoot` (0x319cf735) function
        pub fn last_mainnet_exit_root(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([49, 156, 247, 53], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `lastRollupExitRoot` (0x01fd9044) function
        pub fn last_rollup_exit_root(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([1, 253, 144, 68], ())
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
        ///Calls the contract's `updateExitRoot` (0x33d6247d) function
        pub fn update_exit_root(
            &self,
            new_root: [u8; 32],
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([51, 214, 36, 125], new_root)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `verifyMerkleProof` (0xfb570834) function
        pub fn verify_merkle_proof(
            &self,
            leaf_hash: [u8; 32],
            smt_proof: [[u8; 32]; 32],
            index: u32,
            root: [u8; 32],
        ) -> ::ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([251, 87, 8, 52], (leaf_hash, smt_proof, index, root))
                .expect("method not found (this should never happen)")
        }
        ///Gets the contract's `InitL1InfoRootMap` event
        pub fn init_l1_info_root_map_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            InitL1InfoRootMapFilter,
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
        ///Gets the contract's `UpdateL1InfoTree` event
        pub fn update_l1_info_tree_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            UpdateL1InfoTreeFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `UpdateL1InfoTreeV2` event
        pub fn update_l1_info_tree_v2_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            UpdateL1InfoTreeV2Filter,
        > {
            self.0.event()
        }
        /// Returns an `Event` builder for all the events of this contract.
        pub fn events(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            PolygonZkEVMGlobalExitRootV2Events,
        > {
            self.0.event_with_filter(::core::default::Default::default())
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
    for PolygonZkEVMGlobalExitRootV2<M> {
        fn from(contract: ::ethers::contract::Contract<M>) -> Self {
            Self::new(contract.address(), contract.client())
        }
    }
    ///Custom Error type `GlobalExitRootAlreadySet` with signature `GlobalExitRootAlreadySet()` and selector `0x1f97a582`
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
    #[etherror(name = "GlobalExitRootAlreadySet", abi = "GlobalExitRootAlreadySet()")]
    pub struct GlobalExitRootAlreadySet;
    ///Custom Error type `GlobalExitRootNotFound` with signature `GlobalExitRootNotFound()` and selector `0xf4a66f9d`
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
    #[etherror(name = "GlobalExitRootNotFound", abi = "GlobalExitRootNotFound()")]
    pub struct GlobalExitRootNotFound;
    ///Custom Error type `MerkleTreeFull` with signature `MerkleTreeFull()` and selector `0xef5ccf66`
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
    #[etherror(name = "MerkleTreeFull", abi = "MerkleTreeFull()")]
    pub struct MerkleTreeFull;
    ///Custom Error type `OnlyAllowedContracts` with signature `OnlyAllowedContracts()` and selector `0xb49365dd`
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
    #[etherror(name = "OnlyAllowedContracts", abi = "OnlyAllowedContracts()")]
    pub struct OnlyAllowedContracts;
    ///Custom Error type `OnlyGlobalExitRootRemover` with signature `OnlyGlobalExitRootRemover()` and selector `0xa34ddeb1`
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
    #[etherror(name = "OnlyGlobalExitRootRemover", abi = "OnlyGlobalExitRootRemover()")]
    pub struct OnlyGlobalExitRootRemover;
    ///Custom Error type `OnlyGlobalExitRootUpdater` with signature `OnlyGlobalExitRootUpdater()` and selector `0xc758fc1a`
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
    #[etherror(name = "OnlyGlobalExitRootUpdater", abi = "OnlyGlobalExitRootUpdater()")]
    pub struct OnlyGlobalExitRootUpdater;
    ///Container type for all of the contract's custom errors
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum PolygonZkEVMGlobalExitRootV2Errors {
        GlobalExitRootAlreadySet(GlobalExitRootAlreadySet),
        GlobalExitRootNotFound(GlobalExitRootNotFound),
        MerkleTreeFull(MerkleTreeFull),
        OnlyAllowedContracts(OnlyAllowedContracts),
        OnlyGlobalExitRootRemover(OnlyGlobalExitRootRemover),
        OnlyGlobalExitRootUpdater(OnlyGlobalExitRootUpdater),
        /// The standard solidity revert string, with selector
        /// Error(string) -- 0x08c379a0
        RevertString(::std::string::String),
    }
    impl ::ethers::core::abi::AbiDecode for PolygonZkEVMGlobalExitRootV2Errors {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded) = <::std::string::String as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::RevertString(decoded));
            }
            if let Ok(decoded) = <GlobalExitRootAlreadySet as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GlobalExitRootAlreadySet(decoded));
            }
            if let Ok(decoded) = <GlobalExitRootNotFound as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GlobalExitRootNotFound(decoded));
            }
            if let Ok(decoded) = <MerkleTreeFull as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::MerkleTreeFull(decoded));
            }
            if let Ok(decoded) = <OnlyAllowedContracts as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::OnlyAllowedContracts(decoded));
            }
            if let Ok(decoded) = <OnlyGlobalExitRootRemover as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::OnlyGlobalExitRootRemover(decoded));
            }
            if let Ok(decoded) = <OnlyGlobalExitRootUpdater as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::OnlyGlobalExitRootUpdater(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for PolygonZkEVMGlobalExitRootV2Errors {
        fn encode(self) -> ::std::vec::Vec<u8> {
            match self {
                Self::GlobalExitRootAlreadySet(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GlobalExitRootNotFound(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::MerkleTreeFull(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::OnlyAllowedContracts(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::OnlyGlobalExitRootRemover(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::OnlyGlobalExitRootUpdater(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RevertString(s) => ::ethers::core::abi::AbiEncode::encode(s),
            }
        }
    }
    impl ::ethers::contract::ContractRevert for PolygonZkEVMGlobalExitRootV2Errors {
        fn valid_selector(selector: [u8; 4]) -> bool {
            match selector {
                [0x08, 0xc3, 0x79, 0xa0] => true,
                _ if selector
                    == <GlobalExitRootAlreadySet as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <GlobalExitRootNotFound as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <MerkleTreeFull as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <OnlyAllowedContracts as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <OnlyGlobalExitRootRemover as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <OnlyGlobalExitRootUpdater as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ => false,
            }
        }
    }
    impl ::core::fmt::Display for PolygonZkEVMGlobalExitRootV2Errors {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::GlobalExitRootAlreadySet(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::GlobalExitRootNotFound(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::MerkleTreeFull(element) => ::core::fmt::Display::fmt(element, f),
                Self::OnlyAllowedContracts(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::OnlyGlobalExitRootRemover(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::OnlyGlobalExitRootUpdater(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RevertString(s) => ::core::fmt::Display::fmt(s, f),
            }
        }
    }
    impl ::core::convert::From<::std::string::String>
    for PolygonZkEVMGlobalExitRootV2Errors {
        fn from(value: String) -> Self {
            Self::RevertString(value)
        }
    }
    impl ::core::convert::From<GlobalExitRootAlreadySet>
    for PolygonZkEVMGlobalExitRootV2Errors {
        fn from(value: GlobalExitRootAlreadySet) -> Self {
            Self::GlobalExitRootAlreadySet(value)
        }
    }
    impl ::core::convert::From<GlobalExitRootNotFound>
    for PolygonZkEVMGlobalExitRootV2Errors {
        fn from(value: GlobalExitRootNotFound) -> Self {
            Self::GlobalExitRootNotFound(value)
        }
    }
    impl ::core::convert::From<MerkleTreeFull> for PolygonZkEVMGlobalExitRootV2Errors {
        fn from(value: MerkleTreeFull) -> Self {
            Self::MerkleTreeFull(value)
        }
    }
    impl ::core::convert::From<OnlyAllowedContracts>
    for PolygonZkEVMGlobalExitRootV2Errors {
        fn from(value: OnlyAllowedContracts) -> Self {
            Self::OnlyAllowedContracts(value)
        }
    }
    impl ::core::convert::From<OnlyGlobalExitRootRemover>
    for PolygonZkEVMGlobalExitRootV2Errors {
        fn from(value: OnlyGlobalExitRootRemover) -> Self {
            Self::OnlyGlobalExitRootRemover(value)
        }
    }
    impl ::core::convert::From<OnlyGlobalExitRootUpdater>
    for PolygonZkEVMGlobalExitRootV2Errors {
        fn from(value: OnlyGlobalExitRootUpdater) -> Self {
            Self::OnlyGlobalExitRootUpdater(value)
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
    #[ethevent(name = "InitL1InfoRootMap", abi = "InitL1InfoRootMap(uint32,bytes32)")]
    pub struct InitL1InfoRootMapFilter {
        pub leaf_count: u32,
        pub current_l1_info_root: [u8; 32],
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
    #[ethevent(name = "UpdateL1InfoTree", abi = "UpdateL1InfoTree(bytes32,bytes32)")]
    pub struct UpdateL1InfoTreeFilter {
        #[ethevent(indexed)]
        pub mainnet_exit_root: [u8; 32],
        #[ethevent(indexed)]
        pub rollup_exit_root: [u8; 32],
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
        name = "UpdateL1InfoTreeV2",
        abi = "UpdateL1InfoTreeV2(bytes32,uint32,uint256,uint64)"
    )]
    pub struct UpdateL1InfoTreeV2Filter {
        pub current_l1_info_root: [u8; 32],
        #[ethevent(indexed)]
        pub leaf_count: u32,
        pub blockhash: ::ethers::core::types::U256,
        pub min_timestamp: u64,
    }
    ///Container type for all of the contract's events
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum PolygonZkEVMGlobalExitRootV2Events {
        InitL1InfoRootMapFilter(InitL1InfoRootMapFilter),
        InitializedFilter(InitializedFilter),
        UpdateL1InfoTreeFilter(UpdateL1InfoTreeFilter),
        UpdateL1InfoTreeV2Filter(UpdateL1InfoTreeV2Filter),
    }
    impl ::ethers::contract::EthLogDecode for PolygonZkEVMGlobalExitRootV2Events {
        fn decode_log(
            log: &::ethers::core::abi::RawLog,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::Error> {
            if let Ok(decoded) = InitL1InfoRootMapFilter::decode_log(log) {
                return Ok(
                    PolygonZkEVMGlobalExitRootV2Events::InitL1InfoRootMapFilter(decoded),
                );
            }
            if let Ok(decoded) = InitializedFilter::decode_log(log) {
                return Ok(
                    PolygonZkEVMGlobalExitRootV2Events::InitializedFilter(decoded),
                );
            }
            if let Ok(decoded) = UpdateL1InfoTreeFilter::decode_log(log) {
                return Ok(
                    PolygonZkEVMGlobalExitRootV2Events::UpdateL1InfoTreeFilter(decoded),
                );
            }
            if let Ok(decoded) = UpdateL1InfoTreeV2Filter::decode_log(log) {
                return Ok(
                    PolygonZkEVMGlobalExitRootV2Events::UpdateL1InfoTreeV2Filter(decoded),
                );
            }
            Err(::ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::core::fmt::Display for PolygonZkEVMGlobalExitRootV2Events {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::InitL1InfoRootMapFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InitializedFilter(element) => ::core::fmt::Display::fmt(element, f),
                Self::UpdateL1InfoTreeFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::UpdateL1InfoTreeV2Filter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
            }
        }
    }
    impl ::core::convert::From<InitL1InfoRootMapFilter>
    for PolygonZkEVMGlobalExitRootV2Events {
        fn from(value: InitL1InfoRootMapFilter) -> Self {
            Self::InitL1InfoRootMapFilter(value)
        }
    }
    impl ::core::convert::From<InitializedFilter>
    for PolygonZkEVMGlobalExitRootV2Events {
        fn from(value: InitializedFilter) -> Self {
            Self::InitializedFilter(value)
        }
    }
    impl ::core::convert::From<UpdateL1InfoTreeFilter>
    for PolygonZkEVMGlobalExitRootV2Events {
        fn from(value: UpdateL1InfoTreeFilter) -> Self {
            Self::UpdateL1InfoTreeFilter(value)
        }
    }
    impl ::core::convert::From<UpdateL1InfoTreeV2Filter>
    for PolygonZkEVMGlobalExitRootV2Events {
        fn from(value: UpdateL1InfoTreeV2Filter) -> Self {
            Self::UpdateL1InfoTreeV2Filter(value)
        }
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
    ///Container type for all input parameters for the `calculateRoot` function with signature `calculateRoot(bytes32,bytes32[32],uint32)` and selector `0x83f24403`
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
    #[ethcall(name = "calculateRoot", abi = "calculateRoot(bytes32,bytes32[32],uint32)")]
    pub struct CalculateRootCall {
        pub leaf_hash: [u8; 32],
        pub smt_proof: [[u8; 32]; 32],
        pub index: u32,
    }
    ///Container type for all input parameters for the `depositCount` function with signature `depositCount()` and selector `0x2dfdf0b5`
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
    #[ethcall(name = "depositCount", abi = "depositCount()")]
    pub struct DepositCountCall;
    ///Container type for all input parameters for the `getLastGlobalExitRoot` function with signature `getLastGlobalExitRoot()` and selector `0x3ed691ef`
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
    #[ethcall(name = "getLastGlobalExitRoot", abi = "getLastGlobalExitRoot()")]
    pub struct GetLastGlobalExitRootCall;
    ///Container type for all input parameters for the `getLeafValue` function with signature `getLeafValue(bytes32,uint256,uint64)` and selector `0x5d810501`
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
    #[ethcall(name = "getLeafValue", abi = "getLeafValue(bytes32,uint256,uint64)")]
    pub struct GetLeafValueCall {
        pub new_global_exit_root: [u8; 32],
        pub last_block_hash: ::ethers::core::types::U256,
        pub timestamp: u64,
    }
    ///Container type for all input parameters for the `getRoot` function with signature `getRoot()` and selector `0x5ca1e165`
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
    #[ethcall(name = "getRoot", abi = "getRoot()")]
    pub struct GetRootCall;
    ///Container type for all input parameters for the `globalExitRootMap` function with signature `globalExitRootMap(bytes32)` and selector `0x257b3632`
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
    #[ethcall(name = "globalExitRootMap", abi = "globalExitRootMap(bytes32)")]
    pub struct GlobalExitRootMapCall(pub [u8; 32]);
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
    ///Container type for all input parameters for the `l1InfoRootMap` function with signature `l1InfoRootMap(uint32)` and selector `0xef4eeb35`
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
    #[ethcall(name = "l1InfoRootMap", abi = "l1InfoRootMap(uint32)")]
    pub struct L1InfoRootMapCall {
        pub leaf_count: u32,
    }
    ///Container type for all input parameters for the `lastMainnetExitRoot` function with signature `lastMainnetExitRoot()` and selector `0x319cf735`
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
    #[ethcall(name = "lastMainnetExitRoot", abi = "lastMainnetExitRoot()")]
    pub struct LastMainnetExitRootCall;
    ///Container type for all input parameters for the `lastRollupExitRoot` function with signature `lastRollupExitRoot()` and selector `0x01fd9044`
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
    #[ethcall(name = "lastRollupExitRoot", abi = "lastRollupExitRoot()")]
    pub struct LastRollupExitRootCall;
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
    ///Container type for all input parameters for the `updateExitRoot` function with signature `updateExitRoot(bytes32)` and selector `0x33d6247d`
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
    #[ethcall(name = "updateExitRoot", abi = "updateExitRoot(bytes32)")]
    pub struct UpdateExitRootCall {
        pub new_root: [u8; 32],
    }
    ///Container type for all input parameters for the `verifyMerkleProof` function with signature `verifyMerkleProof(bytes32,bytes32[32],uint32,bytes32)` and selector `0xfb570834`
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
        name = "verifyMerkleProof",
        abi = "verifyMerkleProof(bytes32,bytes32[32],uint32,bytes32)"
    )]
    pub struct VerifyMerkleProofCall {
        pub leaf_hash: [u8; 32],
        pub smt_proof: [[u8; 32]; 32],
        pub index: u32,
        pub root: [u8; 32],
    }
    ///Container type for all of the contract's call
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum PolygonZkEVMGlobalExitRootV2Calls {
        BridgeAddress(BridgeAddressCall),
        CalculateRoot(CalculateRootCall),
        DepositCount(DepositCountCall),
        GetLastGlobalExitRoot(GetLastGlobalExitRootCall),
        GetLeafValue(GetLeafValueCall),
        GetRoot(GetRootCall),
        GlobalExitRootMap(GlobalExitRootMapCall),
        Initialize(InitializeCall),
        L1InfoRootMap(L1InfoRootMapCall),
        LastMainnetExitRoot(LastMainnetExitRootCall),
        LastRollupExitRoot(LastRollupExitRootCall),
        RollupManager(RollupManagerCall),
        UpdateExitRoot(UpdateExitRootCall),
        VerifyMerkleProof(VerifyMerkleProofCall),
    }
    impl ::ethers::core::abi::AbiDecode for PolygonZkEVMGlobalExitRootV2Calls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded) = <BridgeAddressCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::BridgeAddress(decoded));
            }
            if let Ok(decoded) = <CalculateRootCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::CalculateRoot(decoded));
            }
            if let Ok(decoded) = <DepositCountCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::DepositCount(decoded));
            }
            if let Ok(decoded) = <GetLastGlobalExitRootCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GetLastGlobalExitRoot(decoded));
            }
            if let Ok(decoded) = <GetLeafValueCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GetLeafValue(decoded));
            }
            if let Ok(decoded) = <GetRootCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GetRoot(decoded));
            }
            if let Ok(decoded) = <GlobalExitRootMapCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GlobalExitRootMap(decoded));
            }
            if let Ok(decoded) = <InitializeCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::Initialize(decoded));
            }
            if let Ok(decoded) = <L1InfoRootMapCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::L1InfoRootMap(decoded));
            }
            if let Ok(decoded) = <LastMainnetExitRootCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::LastMainnetExitRoot(decoded));
            }
            if let Ok(decoded) = <LastRollupExitRootCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::LastRollupExitRoot(decoded));
            }
            if let Ok(decoded) = <RollupManagerCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::RollupManager(decoded));
            }
            if let Ok(decoded) = <UpdateExitRootCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::UpdateExitRoot(decoded));
            }
            if let Ok(decoded) = <VerifyMerkleProofCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::VerifyMerkleProof(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for PolygonZkEVMGlobalExitRootV2Calls {
        fn encode(self) -> Vec<u8> {
            match self {
                Self::BridgeAddress(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CalculateRoot(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::DepositCount(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetLastGlobalExitRoot(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetLeafValue(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetRoot(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::GlobalExitRootMap(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::Initialize(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::L1InfoRootMap(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::LastMainnetExitRoot(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::LastRollupExitRoot(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RollupManager(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::UpdateExitRoot(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::VerifyMerkleProof(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
            }
        }
    }
    impl ::core::fmt::Display for PolygonZkEVMGlobalExitRootV2Calls {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::BridgeAddress(element) => ::core::fmt::Display::fmt(element, f),
                Self::CalculateRoot(element) => ::core::fmt::Display::fmt(element, f),
                Self::DepositCount(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetLastGlobalExitRoot(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::GetLeafValue(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetRoot(element) => ::core::fmt::Display::fmt(element, f),
                Self::GlobalExitRootMap(element) => ::core::fmt::Display::fmt(element, f),
                Self::Initialize(element) => ::core::fmt::Display::fmt(element, f),
                Self::L1InfoRootMap(element) => ::core::fmt::Display::fmt(element, f),
                Self::LastMainnetExitRoot(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::LastRollupExitRoot(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RollupManager(element) => ::core::fmt::Display::fmt(element, f),
                Self::UpdateExitRoot(element) => ::core::fmt::Display::fmt(element, f),
                Self::VerifyMerkleProof(element) => ::core::fmt::Display::fmt(element, f),
            }
        }
    }
    impl ::core::convert::From<BridgeAddressCall> for PolygonZkEVMGlobalExitRootV2Calls {
        fn from(value: BridgeAddressCall) -> Self {
            Self::BridgeAddress(value)
        }
    }
    impl ::core::convert::From<CalculateRootCall> for PolygonZkEVMGlobalExitRootV2Calls {
        fn from(value: CalculateRootCall) -> Self {
            Self::CalculateRoot(value)
        }
    }
    impl ::core::convert::From<DepositCountCall> for PolygonZkEVMGlobalExitRootV2Calls {
        fn from(value: DepositCountCall) -> Self {
            Self::DepositCount(value)
        }
    }
    impl ::core::convert::From<GetLastGlobalExitRootCall>
    for PolygonZkEVMGlobalExitRootV2Calls {
        fn from(value: GetLastGlobalExitRootCall) -> Self {
            Self::GetLastGlobalExitRoot(value)
        }
    }
    impl ::core::convert::From<GetLeafValueCall> for PolygonZkEVMGlobalExitRootV2Calls {
        fn from(value: GetLeafValueCall) -> Self {
            Self::GetLeafValue(value)
        }
    }
    impl ::core::convert::From<GetRootCall> for PolygonZkEVMGlobalExitRootV2Calls {
        fn from(value: GetRootCall) -> Self {
            Self::GetRoot(value)
        }
    }
    impl ::core::convert::From<GlobalExitRootMapCall>
    for PolygonZkEVMGlobalExitRootV2Calls {
        fn from(value: GlobalExitRootMapCall) -> Self {
            Self::GlobalExitRootMap(value)
        }
    }
    impl ::core::convert::From<InitializeCall> for PolygonZkEVMGlobalExitRootV2Calls {
        fn from(value: InitializeCall) -> Self {
            Self::Initialize(value)
        }
    }
    impl ::core::convert::From<L1InfoRootMapCall> for PolygonZkEVMGlobalExitRootV2Calls {
        fn from(value: L1InfoRootMapCall) -> Self {
            Self::L1InfoRootMap(value)
        }
    }
    impl ::core::convert::From<LastMainnetExitRootCall>
    for PolygonZkEVMGlobalExitRootV2Calls {
        fn from(value: LastMainnetExitRootCall) -> Self {
            Self::LastMainnetExitRoot(value)
        }
    }
    impl ::core::convert::From<LastRollupExitRootCall>
    for PolygonZkEVMGlobalExitRootV2Calls {
        fn from(value: LastRollupExitRootCall) -> Self {
            Self::LastRollupExitRoot(value)
        }
    }
    impl ::core::convert::From<RollupManagerCall> for PolygonZkEVMGlobalExitRootV2Calls {
        fn from(value: RollupManagerCall) -> Self {
            Self::RollupManager(value)
        }
    }
    impl ::core::convert::From<UpdateExitRootCall>
    for PolygonZkEVMGlobalExitRootV2Calls {
        fn from(value: UpdateExitRootCall) -> Self {
            Self::UpdateExitRoot(value)
        }
    }
    impl ::core::convert::From<VerifyMerkleProofCall>
    for PolygonZkEVMGlobalExitRootV2Calls {
        fn from(value: VerifyMerkleProofCall) -> Self {
            Self::VerifyMerkleProof(value)
        }
    }
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
    ///Container type for all return fields from the `calculateRoot` function with signature `calculateRoot(bytes32,bytes32[32],uint32)` and selector `0x83f24403`
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
    pub struct CalculateRootReturn(pub [u8; 32]);
    ///Container type for all return fields from the `depositCount` function with signature `depositCount()` and selector `0x2dfdf0b5`
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
    pub struct DepositCountReturn(pub ::ethers::core::types::U256);
    ///Container type for all return fields from the `getLastGlobalExitRoot` function with signature `getLastGlobalExitRoot()` and selector `0x3ed691ef`
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
    pub struct GetLastGlobalExitRootReturn(pub [u8; 32]);
    ///Container type for all return fields from the `getLeafValue` function with signature `getLeafValue(bytes32,uint256,uint64)` and selector `0x5d810501`
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
    pub struct GetLeafValueReturn(pub [u8; 32]);
    ///Container type for all return fields from the `getRoot` function with signature `getRoot()` and selector `0x5ca1e165`
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
    pub struct GetRootReturn(pub [u8; 32]);
    ///Container type for all return fields from the `globalExitRootMap` function with signature `globalExitRootMap(bytes32)` and selector `0x257b3632`
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
    pub struct GlobalExitRootMapReturn(pub ::ethers::core::types::U256);
    ///Container type for all return fields from the `l1InfoRootMap` function with signature `l1InfoRootMap(uint32)` and selector `0xef4eeb35`
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
    pub struct L1InfoRootMapReturn {
        pub l_1_info_root: [u8; 32],
    }
    ///Container type for all return fields from the `lastMainnetExitRoot` function with signature `lastMainnetExitRoot()` and selector `0x319cf735`
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
    pub struct LastMainnetExitRootReturn(pub [u8; 32]);
    ///Container type for all return fields from the `lastRollupExitRoot` function with signature `lastRollupExitRoot()` and selector `0x01fd9044`
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
    pub struct LastRollupExitRootReturn(pub [u8; 32]);
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
    ///Container type for all return fields from the `verifyMerkleProof` function with signature `verifyMerkleProof(bytes32,bytes32[32],uint32,bytes32)` and selector `0xfb570834`
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
    pub struct VerifyMerkleProofReturn(pub bool);
}
