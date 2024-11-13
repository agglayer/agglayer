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
            ]),
            receive: false,
            fallback: false,
        }
    }
    ///The parsed JSON ABI of the contract.
    pub static POLYGONZKEVMGLOBALEXITROOTV2_ABI: ::ethers::contract::Lazy<
        ::ethers::core::abi::Abi,
    > = ::ethers::contract::Lazy::new(__abi);
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
    ///Container type for all of the contract's custom errors
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum PolygonZkEVMGlobalExitRootV2Errors {
        MerkleTreeFull(MerkleTreeFull),
        OnlyAllowedContracts(OnlyAllowedContracts),
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
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for PolygonZkEVMGlobalExitRootV2Errors {
        fn encode(self) -> ::std::vec::Vec<u8> {
            match self {
                Self::MerkleTreeFull(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::OnlyAllowedContracts(element) => {
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
                    == <MerkleTreeFull as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <OnlyAllowedContracts as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ => false,
            }
        }
    }
    impl ::core::fmt::Display for PolygonZkEVMGlobalExitRootV2Errors {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::MerkleTreeFull(element) => ::core::fmt::Display::fmt(element, f),
                Self::OnlyAllowedContracts(element) => {
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
